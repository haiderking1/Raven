use mlua::{Result, Value};

pub(super) fn value<T>(
    name: &str,
    value: Value,
    parse: impl FnOnce(Value) -> Result<T>,
) -> Result<T> {
    let received = match &value {
        Value::String(s) => {
            let text = s.to_string_lossy();
            let short: String = text.chars().take(80).collect();
            format!(
                "string {short:?}{}",
                if text.chars().count() > 80 {
                    " (shortened)"
                } else {
                    ""
                }
            )
        }
        Value::Integer(n) => n.to_string(),
        Value::Number(n) => format!("number {n:?}"),
        Value::Boolean(b) => b.to_string(),
        other => other.type_name().to_owned(),
    };
    parse(value)
        .map_err(|error| super::values::error(format!("{name}: {error}. Received {received}.")))
}
