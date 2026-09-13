mod appearance;
mod bindings;
mod checked;
mod input;
mod limits;
mod resources;
mod session;
mod values;

use super::Prepared;
use crate::{backend::tty::PreparedCursor, runtime::settings::Settings};
use mlua::{Lua, LuaOptions, StdLib, Value};
use std::{cell::RefCell, path::Path, rc::Rc};

struct Builder {
    settings: Settings,
    cursor: Option<PreparedCursor>,
}
type Build = Rc<RefCell<Builder>>;

pub(super) fn parse(source: &[u8], path: &Path, defaults: Settings) -> Result<Prepared, String> {
    evaluate(source, path, defaults).map_err(super::diagnostic::bounded)
}

fn evaluate(source: &[u8], path: &Path, defaults: Settings) -> Result<Prepared, String> {
    let lua = Lua::new_with(
        StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
        LuaOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    let line = limits::install(&lua).map_err(|e| e.to_string())?;
    for name in ["dofile", "loadfile", "load", "collectgarbage"] {
        lua.globals()
            .set(name, Value::Nil)
            .map_err(|e| e.to_string())?;
    }
    let build = Rc::new(RefCell::new(Builder {
        settings: defaults,
        cursor: None,
    }));
    let result = (|| -> mlua::Result<()> {
        let api = lua.create_table()?;
        api.set(
            "config_dir",
            path.parent()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
                .as_ref(),
        )?;
        api.set(
            "env",
            lua.create_function(|_, name: String| {
                if name.contains(char::from(0)) || name.contains('=') {
                    return Err(values::error("invalid environment name"));
                }
                Ok(std::env::var(name).ok())
            })?,
        )?;
        appearance::install(&lua, &api, build.clone())?;
        bindings::install(&lua, &api, build.clone())?;
        session::install(&lua, &api, build.clone())?;
        resources::install(&lua, &api, build.clone())?;
        input::install(&lua, &api, build.clone())?;
        let proxy = lua.create_table()?;
        let meta = lua.create_table()?;
        meta.set("__index", api)?;
        meta.set(
            "__newindex",
            lua.create_function(|_, (_, key, _): (Value, Value, Value)| {
                Err::<(), _>(values::error(format!(
                    "raven API is read-only; use a setting function, got {key:?}"
                )))
            })?,
        )?;
        meta.set("__metatable", false)?;
        proxy.set_metatable(Some(meta))?;
        lua.globals().set("raven", proxy)?;
        let result: Value = lua
            .load(source)
            .set_name(format!("@{}", path.display()))
            .eval()?;
        if !matches!(result, Value::Nil) {
            return Err(values::error(
                "configure Raven with raven.* calls, not a returned table",
            ));
        }
        Ok(())
    })();
    result.map_err(|e| super::diagnostic::report(&lua, &e, path, source, line.get()))?;
    // Dropping Lua releases callback captures before unwrapping the builder.
    drop(lua);
    let Builder { settings, cursor } = Rc::try_unwrap(build)
        .map_err(|_| "configuration builder still referenced")?
        .into_inner();
    settings.validate()?;
    // Also cover inherited defaults when the file omits raven.commands.
    for (name, argv) in [
        ("commands.terminal", &settings.terminal),
        ("commands.launcher", &settings.launcher),
    ] {
        super::commands::validate(name, argv)
            .map_err(|error| format!("{}\n\n{error}", path.display()))?;
    }
    let cursor = cursor
        .map(Ok)
        .unwrap_or_else(|| PreparedCursor::load(settings.cursor.clone()))
        .map_err(|e| e.to_string())?;
    Ok(Prepared { settings, cursor })
}
