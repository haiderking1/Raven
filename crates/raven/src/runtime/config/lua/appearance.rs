use super::{Build, checked::value as checked, values::*};
use mlua::{Lua, Result, Table, Value};

fn extent(value: Value) -> Result<i32> {
    let n = integer(value)?;
    if !(0..=65_535).contains(&n) {
        return Err(error(
            "use a whole number from 0 through 65535; use 0 to disable this gap or border",
        ));
    }
    Ok(n as i32)
}

pub(super) fn install(lua: &Lua, api: &Table, build: Build) -> Result<()> {
    let appearance = build.clone();
    api.set(
        "appearance",
        lua.create_function(move |_, table: Table| {
            fields(&table, &["inner", "outer", "border"])?;
            let mut value = appearance.borrow().settings.appearance;
            match table.get::<Value>("inner")? {
                Value::Nil => {}
                Value::Table(inner) => {
                    fields(&inner, &["horizontal", "vertical"])?;
                    if let Some(v) = inner.get::<Option<Value>>("horizontal")? {
                        value.inner.horizontal = checked("appearance.inner.horizontal", v, extent)?;
                    }
                    if let Some(v) = inner.get::<Option<Value>>("vertical")? {
                        value.inner.vertical = checked("appearance.inner.vertical", v, extent)?;
                    }
                }
                v => {
                    let n = checked("appearance.inner", v, extent)?;
                    value.inner.horizontal = n;
                    value.inner.vertical = n;
                }
            }
            match table.get::<Value>("outer")? {
                Value::Nil => {}
                Value::Table(outer) => {
                    fields(&outer, &["top", "right", "bottom", "left"])?;
                    for (name, field) in [
                        ("top", &mut value.outer.top),
                        ("right", &mut value.outer.right),
                        ("bottom", &mut value.outer.bottom),
                        ("left", &mut value.outer.left),
                    ] {
                        if let Some(v) = outer.get::<Option<Value>>(name)? {
                            *field = checked(&format!("appearance.outer.{name}"), v, extent)?;
                        }
                    }
                }
                v => {
                    let n = checked("appearance.outer", v, extent)?;
                    value.outer = crate::runtime::settings::OuterGaps {
                        top: n,
                        right: n,
                        bottom: n,
                        left: n,
                    };
                }
            }
            if let Some(border) = table.get::<Option<Value>>("border")? {
                let border = checked("appearance.border", border, super::values::table)?;
                fields(&border, &["width", "active", "inactive"])?;
                if let Some(v) = border.get::<Option<Value>>("width")? {
                    value.border.width = checked("appearance.border.width", v, extent)?;
                }
                if let Some(v) = border.get::<Option<Value>>("active")? {
                    value.border.active = checked("appearance.border.active", v, color)?;
                }
                if let Some(v) = border.get::<Option<Value>>("inactive")? {
                    value.border.inactive = checked("appearance.border.inactive", v, color)?;
                }
            }
            value.validate().map_err(|e| error(e.to_string()))?;
            appearance.borrow_mut().settings.appearance = value;
            Ok(())
        })?,
    )?;
    api.set(
        "animations",
        lua.create_function(move |_, table: Table| {
            fields(&table, &["resize_ms"])?;
            if let Some(v) = table.get::<Option<Value>>("resize_ms")? {
                let n = checked("animations.resize_ms", v, integer)?;
                if !(0..=2000).contains(&n) {
                    return Err(error(format!("animations.resize_ms: use a whole number from 0 through 2000 milliseconds; 0 disables animation. Received {n}.")));
                }
                build.borrow_mut().settings.resize_animations =
                    crate::runtime::settings::ResizeAnimations::from_millis(n as u64)
                        .map_err(|e| error(e.to_string()))?;
            }
            Ok(())
        })?,
    )
}
