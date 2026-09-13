mod repair;
mod report;
mod source;
mod syntax;
pub(super) use report::{Validation, report};

/// A Lua error may contain an arbitrarily large user string. Bound it on the
/// worker before the compositor copies it or passes it to the error process.
pub(super) fn bounded(mut message: String) -> String {
    const LIMIT: usize = 64 * 1024;
    if message.len() > LIMIT {
        let mut end = LIMIT;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push_str("\n\n[Diagnostic truncated at 64 KiB]");
    }
    message
}
