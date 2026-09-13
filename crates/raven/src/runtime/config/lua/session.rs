use super::{Build, checked::value as checked, values::*};
use crate::runtime::settings::{StartupEntry, StartupPlan};
use mlua::{Lua, Result, Table, Value};

pub(super) fn install(lua: &Lua, api: &Table, build: Build) -> Result<()> {
    let target = build.clone();
    api.set(
        "commands",
        lua.create_function(move |_, table: Table| {
            fields(&table, &["terminal", "launcher"])?;
            let mut state = target.borrow_mut();
            if let Some(v) = table.get::<Option<Value>>("terminal")? {
                let command = checked("commands.terminal", v, argv)?;
                super::super::commands::validate("commands.terminal", &command).map_err(error)?;
                state.settings.terminal = command;
            }
            if let Some(v) = table.get::<Option<Value>>("launcher")? {
                let command = checked("commands.launcher", v, argv)?;
                super::super::commands::validate("commands.launcher", &command).map_err(error)?;
                state.settings.launcher = command;
            }
            Ok(())
        })?,
    )?;
    api.set(
        "startup",
        lua.create_function(move |_, table: Table| {
            let mut entries = Vec::new();
            for value in sequence(table)? {
                let table = self::table(value)?;
                let entry = if table.contains_key("argv")? {
                    fields(&table, &["argv", "cwd", "env"])?;
                    let mut entry = StartupEntry {
                        argv: argv(table.get("argv")?)?,
                        ..Default::default()
                    };
                    if let Some(v) = table.get::<Option<Value>>("cwd")? {
                        entry.cwd = Some(text(v)?.into());
                    }
                    if let Some(env) = table.get::<Option<Table>>("env")? {
                        for pair in env.pairs::<Value, Value>() {
                            let (k, v) = pair?;
                            entry.env.insert(text(k)?.into(), text(v)?.into());
                        }
                    }
                    entry
                } else {
                    StartupEntry {
                        argv: argv(Value::Table(table))?,
                        ..Default::default()
                    }
                };
                entries.push(entry);
            }
            let plan = StartupPlan { entries };
            plan.validate_all().map_err(error)?;
            build.borrow_mut().settings.startup = plan;
            Ok(())
        })?,
    )
}
