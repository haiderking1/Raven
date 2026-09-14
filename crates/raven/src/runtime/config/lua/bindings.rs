use super::{Build, values::*};
use crate::runtime::settings::Action;
use mlua::{Lua, Result, Table, Value};

fn action(value: Value, argument: Option<i64>) -> Result<Action> {
    if let Value::Table(value) = value {
        fields(&value, &["spawn"])?;
        if argument.is_some() {
            return Err(error("spawn does not accept an action parameter"));
        }
        return Ok(Action::Spawn(argv(value.get("spawn")?)?));
    }
    let name = text(value)?;
    match name.as_str() {
        "workspace" | "move_to_workspace" | "vt" => {
            let n = argument.ok_or_else(|| error(format!("{name} requires a number")))?;
            let max = if name == "vt" { 12 } else { 10 };
            if !(1..=max).contains(&n) {
                return Err(error(format!("{name} number must be between 1 and {max}")));
            }
            Ok(match name.as_str() {
                "workspace" => Action::SwitchWorkspace(n as usize - 1),
                "move_to_workspace" => Action::MoveToWorkspace(n as usize - 1),
                _ => Action::SwitchVt(n as i32),
            })
        }
        _ => {
            if argument.is_some() {
                return Err(error(format!("{name} does not accept a number")));
            }
            match name.as_str() {
                "quit" => Ok(Action::Quit),
                "terminal" => Ok(Action::LaunchTerminal),
                "launcher" => Ok(Action::LaunchFuzzel),
                "close" => Ok(Action::CloseWindow),
                "fullscreen" => Ok(Action::ToggleFullscreen),
                "floating" => Ok(Action::ToggleFloating),
                "reload" => Ok(Action::ReloadConfig),
                "screenshot" => Ok(Action::Screenshot),
                "next_app" => Ok(Action::CycleApplications(false)),
                "previous_app" => Ok(Action::CycleApplications(true)),
                _ => Err(error(format!(
                    "unknown binding action {name:?}. Choose terminal, launcher, close, fullscreen, floating, screenshot, next_app, previous_app, quit, reload, workspace, move_to_workspace, or vt. To launch a program, use {{ spawn = {{ 'program' }} }}"
                ))),
            }
        }
    }
}

pub(super) fn install(lua: &Lua, api: &Table, build: Build) -> Result<()> {
    let target = build.clone();
    api.set(
        "bind",
        lua.create_function(
            move |_, (chord, value, arg): (String, Value, Option<i64>)| {
                target
                    .borrow_mut()
                    .settings
                    .bindings
                    .bind(&chord, action(value, arg)?)
                    .map_err(error)
            },
        )?,
    )?;
    let target = build.clone();
    api.set(
        "unbind",
        lua.create_function(move |_, chord: String| {
            target
                .borrow_mut()
                .settings
                .bindings
                .unbind(&chord)
                .map_err(error)
        })?,
    )?;
    api.set(
        "clear_bindings",
        lua.create_function(move |_, ()| {
            build.borrow_mut().settings.bindings.clear();
            Ok(())
        })?,
    )
}
