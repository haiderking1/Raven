use mlua::{Error, Result, Table, Value};
use std::ffi::OsString;

pub(super) fn error(message: impl Into<String>) -> Error {
    Error::external(super::super::diagnostic::Validation(message.into()))
}

pub(super) fn fields(table: &Table, names: &[&str]) -> Result<()> {
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        let Value::String(key) = key else {
            return Err(error("setting names must be strings"));
        };
        let key = key.to_str()?;
        if !names.contains(&key.as_ref()) {
            return Err(error(format!(
                "unknown setting {key:?}. Allowed settings here: {}",
                names.join(", ")
            )));
        }
    }
    Ok(())
}

pub(super) fn sequence(table: Table) -> Result<Vec<Value>> {
    let length = table.raw_len();
    let mut count = 0;
    for pair in table.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        if !matches!(key, Value::Integer(n) if n >= 1 && n as usize <= length) {
            return Err(error("expected a dense array starting at index 1"));
        }
        count += 1;
    }
    if count != length {
        return Err(error("array contains missing entries"));
    }
    (1..=length).map(|index| table.raw_get(index)).collect()
}

pub(super) fn text(value: Value) -> Result<String> {
    let Value::String(value) = value else {
        return Err(error("expected text in quotes, for example 'foot'"));
    };
    let value = value.to_str()?.to_string();
    if value.contains(char::from(0)) {
        return Err(error("strings must not contain NUL"));
    }
    Ok(value)
}

pub(super) fn integer(value: Value) -> Result<i64> {
    match value {
        Value::Integer(value) => Ok(value),
        _ => Err(error(
            "expected a whole number without quotes or a decimal point, for example 2",
        )),
    }
}

pub(super) fn table(value: Value) -> Result<Table> {
    match value {
        Value::Table(value) => Ok(value),
        _ => Err(error(
            "expected a list or settings block in braces, for example { 'foot' }",
        )),
    }
}

pub(super) fn argv(value: Value) -> Result<Vec<OsString>> {
    let values = sequence(table(value)?)?;
    let argv = values
        .into_iter()
        .map(|value| text(value).map(OsString::from))
        .collect::<Result<Vec<_>>>()?;
    if argv.first().is_none_or(|program| program.is_empty()) {
        return Err(error(
            "command must be a nonempty argv array, for example { 'foot' }",
        ));
    }
    Ok(argv)
}

pub(super) fn color(value: Value) -> Result<[f32; 4]> {
    let text = text(value)?;
    let hex = text
        .strip_prefix('#')
        .ok_or_else(|| error("color must be #RRGGBB or #RRGGBBAA"))?;
    if !matches!(hex.len(), 6 | 8) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(error("color must be #RRGGBB or #RRGGBBAA"));
    }
    let mut rgba = [1.0; 4];
    for (index, channel) in rgba.iter_mut().enumerate().take(hex.len() / 2) {
        *channel = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap() as f32 / 255.0;
    }
    Ok(rgba)
}
