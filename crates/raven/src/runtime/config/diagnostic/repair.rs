use mlua::Lua;

pub(super) struct Suggestion {
    pub line: usize,
    pub name: String,
}

fn identifier(text: &str) -> bool {
    let mut bytes = text.bytes();
    matches!(bytes.next(), Some(c) if c.is_ascii_alphabetic() || c == b'_')
        && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
        && ![
            "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto",
            "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until",
            "while",
        ]
        .contains(&text)
}

/// A suggestion, not an edit: removing a name can also change the meaning of a
/// multiline expression. Only offer it when Lua confirms the entire file parses.
/// Compile on the existing memory-limited VM, without executing any code or API.
pub(super) fn suggest(lua: &Lua, source: &[u8], detected: Option<usize>) -> Option<Suggestion> {
    let detected = detected?;
    let text = std::str::from_utf8(source).ok()?;
    let mut offset = 0;
    let mut candidates = Vec::new();
    for (index, line) in text.split_inclusive('\n').enumerate().take(detected) {
        let number = index + 1;
        let name = line.trim();
        if number < detected && number >= detected.saturating_sub(4) && identifier(name) {
            candidates.push((number, offset, line.len(), name));
        }
        offset += line.len();
    }
    for (line, start, length, name) in candidates.into_iter().rev() {
        let mut candidate = Vec::with_capacity(source.len());
        candidate.extend_from_slice(&source[..start]);
        candidate.extend_from_slice(&source[start + length..]);
        if lua
            .load(&candidate)
            .set_name("=diagnostic suggestion")
            .into_function()
            .is_ok()
        {
            return Some(Suggestion {
                line,
                name: name.to_owned(),
            });
        }
    }
    None
}
