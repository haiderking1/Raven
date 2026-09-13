use super::{Build, checked::value as checked, values::*};
use crate::backend::tty::PreparedCursor;
use mlua::{Lua, Result, Table, Value};

pub(super) fn install(lua: &Lua, api: &Table, build: Build) -> Result<()> {
    let target = build.clone();
    api.set(
        "cursor",
        lua.create_function(move |_, table: Table| {
            fields(&table, &["theme", "size"])?;
            let mut settings = target.borrow().settings.cursor.clone();
            if let Some(v) = table.get::<Option<Value>>("theme")? {
                settings.theme = checked("cursor.theme", v, text)?;
            }
            if let Some(v) = table.get::<Option<Value>>("size")? {
                let n = checked("cursor.size", v, integer)?;
                if !(1..=i32::MAX as i64).contains(&n) {
                    return Err(error(format!("cursor.size: use a whole number from 1 through 2147483647 logical pixels, for example 24. Received {n}.")));
                }
                settings.size = n as u32;
            }
            let prepared =
                PreparedCursor::load(settings.clone()).map_err(|e| error(e.to_string()))?;
            let mut target = target.borrow_mut();
            target.settings.cursor = settings;
            target.cursor = Some(prepared);
            Ok(())
        })?,
    )?;
    api.set(
        "workspaces",
        lua.create_function(move |_, table: Table| {
            fields(&table, &["show", "persistent"])?;
            let mut settings = build.borrow().settings.workspaces.clone();
            if let Some(v) = table.get::<Option<Value>>("show")? {
                settings.show_all = match checked("workspaces.show", v, text)?.as_str() {
                    "all" => true,
                    "occupied" => false,
                    other => return Err(error(format!("workspaces.show: choose 'all' or 'occupied'. Received {other:?}."))),
                };
            }
            if let Some(list) = table.get::<Option<Table>>("persistent")? {
                settings.persistent = sequence(list)?
                    .into_iter()
                    .enumerate()
                    .map(|(index, v)| {
                        let n = checked(&format!("workspaces.persistent[{}]", index + 1), v, integer)?;
                        if !(1..=10).contains(&n) {
                            return Err(error(
                                format!("workspaces.persistent[{}]: choose a workspace from 1 through 10. Received {n}.", index + 1),
                            ));
                        }
                        Ok(n as usize)
                    })
                    .collect::<Result<Vec<_>>>()?;
            }
            build.borrow_mut().settings.workspaces = settings;
            Ok(())
        })?,
    )
}
