pub(super) fn help(message: &str) -> &'static str {
    if message.contains("unfinished string") {
        "A quoted string was not closed. Put matching quotes around text, for example terminal = { 'foot' }. Check the preceding line too."
    } else if message.contains("unfinished long") {
        "A multiline string or comment was not closed. Match its opening [[ with ]], or [=[ with ]=]."
    } else if message.contains("'end' expected") {
        "A function, loop, or if block needs a matching end keyword. Check the block opening mentioned in Lua's message."
    } else if message.contains("'then' expected") {
        "An if condition needs then before its body, for example: if enabled then ... end."
    } else if message.contains("<eof>") {
        "Lua reached the end of the file. Check for a missing closing brace, parenthesis, quote, or end keyword."
    } else if message.contains("expected") || message.contains("unexpected symbol") {
        "Check for a missing comma between entries and matching braces or parentheses. Example: border = { width = 2, active = '#228B22' }. Start with the line before Lua's reported location."
    } else {
        "Check spelling, matching quotes/brackets, and commas between table entries near the reported location. Save the corrected file to retry."
    }
}
