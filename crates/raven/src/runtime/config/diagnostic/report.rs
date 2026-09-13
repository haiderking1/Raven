use mlua::Error;
use std::{fmt, path::Path};

#[derive(Debug)]
pub(in crate::runtime::config) struct Validation(pub String);
impl fmt::Display for Validation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Validation {}

fn cause(error: &Error) -> &Error {
    match error {
        Error::CallbackError { cause: inner, .. }
        | Error::BadArgument { cause: inner, .. }
        | Error::WithContext { cause: inner, .. } => cause(inner),
        _ => error,
    }
}

pub(in crate::runtime::config) fn report(
    lua: &mlua::Lua,
    error: &Error,
    path: &Path,
    source: &[u8],
    line: Option<usize>,
) -> String {
    let root = cause(error);
    let syntax = matches!(root, Error::SyntaxError { .. });
    let validation = matches!(root, Error::ExternalError(e) if e.is::<Validation>())
        || matches!(root, Error::FromLuaConversionError { .. });
    let title = if syntax {
        "Lua could not read this configuration"
    } else if validation {
        "A configuration setting is invalid"
    } else {
        "The configuration stopped while running"
    };
    let detail = match root {
        Error::SyntaxError { message, .. } | Error::RuntimeError(message) => message.clone(),
        _ => root.to_string(),
    };
    let line = if syntax {
        super::source::reported_line(&detail)
    } else {
        line
    };
    if syntax && let Some(suggestion) = super::repair::suggest(lua, source, line) {
        return format!(
            "Possible stray name {name:?} on line {suspected}.

Removing that line makes the file valid Lua. If the text was accidental, delete it. If it was intentional, complete its assignment or function call instead. This suggestion was checked by compiling, not running, a temporary copy. Your file was not changed.

{path}:{suspected}
Suggested place to check, not a guaranteed correction:
{excerpt}
Lua detected the problem later, on line {detected}. That is not necessarily the line to edit.

Technical details (included by Copy error):
{error}",
            name = suggestion.name, suspected = suggestion.line, path = path.display(),
            excerpt = super::source::excerpt(source, Some(suggestion.line)),
            detected = line.expect("a suggestion requires a detected line"),
        );
    }
    let location = match line {
        Some(n) => format!("{}:{n}", path.display()),
        None => path.display().to_string(),
    };
    let explanation = if syntax {
        super::syntax::help(&detail)
    } else if validation {
        "Change the setting described above, then save the file. Examples are not automatic replacements."
    } else {
        "Check the expression or function at the reported location. Fix it and save the file to retry."
    };
    let location_note = if syntax {
        "Lua detected the problem here. A missing comma, quote, or closing bracket may be on an earlier line."
    } else {
        "Reported execution/call location. For a table of settings, the invalid field may be on another line in that table."
    };
    format!(
        "{title}

{detail}

{explanation}

{location}
{location_note}
{}
Technical details (included by Copy error):
{error}",
        super::source::excerpt(source, line)
    )
}
