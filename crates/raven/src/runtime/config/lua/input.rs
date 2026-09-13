use super::{Build, checked::value as checked, values::*};
use crate::runtime::settings::MouseAccelProfile;
use mlua::{Lua, Result, Table, Value};

pub(super) fn install(lua: &Lua, api: &Table, build: Build) -> Result<()> {
    api.set(
        "input",
        lua.create_function(move |_, table: Table| {
            fields(&table, &["keyboard", "mouse"])?;
            let mut settings = build.borrow().settings.input;
            if let Some(keyboard) = table.get::<Option<Table>>("keyboard")? {
                fields(&keyboard, &["repeat_rate", "repeat_delay"])?;
                for (name, target, max, unit) in [
                    (
                        "repeat_rate",
                        &mut settings.keyboard.repeat_rate,
                        1000,
                        "repeats per second",
                    ),
                    (
                        "repeat_delay",
                        &mut settings.keyboard.repeat_delay,
                        60000,
                        "milliseconds",
                    ),
                ] {
                    if let Some(value) = keyboard.get::<Option<Value>>(name)? {
                        *target = checked(&format!("input.keyboard.{name}"), value, |v| {
                            let n = integer(v)?;
                            if !(0..=max).contains(&n) {
                                return Err(error(format!(
                                    "use a whole number from 0 to {max} {unit}"
                                )));
                            }
                            Ok(n as i32)
                        })?;
                    }
                }
            }
            if let Some(mouse) = table.get::<Option<Table>>("mouse")? {
                fields(&mouse, &["accel_profile"])?;
                if let Some(value) = mouse.get::<Option<Value>>("accel_profile")? {
                    settings.mouse_accel_profile =
                        checked("input.mouse.accel_profile", value, |v| {
                            match text(v)?.as_str() {
                                "default" => Ok(MouseAccelProfile::Default),
                                "flat" => Ok(MouseAccelProfile::Flat),
                                "adaptive" => Ok(MouseAccelProfile::Adaptive),
                                _ => Err(error("use 'default', 'flat', or 'adaptive'")),
                            }
                        })?;
                }
            }
            settings.validate().map_err(error)?;
            build.borrow_mut().settings.input = settings;
            Ok(())
        })?,
    )
}
