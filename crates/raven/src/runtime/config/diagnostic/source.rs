pub(super) fn reported_line(message: &str) -> Option<usize> {
    // Lua syntax errors begin with chunk-name:line:message.
    message
        .lines()
        .next()?
        .split(':')
        .skip(1)
        .find_map(|part| part.parse::<usize>().ok().filter(|n| *n > 0))
}

pub(super) fn excerpt(source: &[u8], line: Option<usize>) -> String {
    let Some(line) = line.filter(|n| *n > 0) else {
        return String::new();
    };
    let text = String::from_utf8_lossy(source);
    let start = line.saturating_sub(4).max(1);
    let end = line.saturating_add(2);
    let mut result = String::new();
    for (index, content) in text.lines().enumerate().take(end).skip(start - 1) {
        let number = index + 1;
        let mark = if number == line { ">" } else { " " };
        let mut displayed: String = content
            .chars()
            .take(240)
            .flat_map(|c| {
                if c.is_control() {
                    c.escape_default().collect::<Vec<_>>()
                } else {
                    vec![c]
                }
            })
            .collect();
        if content.chars().count() > 240 {
            displayed.push_str(" ... [line shortened]");
        }
        result.push_str(&format!(
            "{mark} {number:>4} | {displayed}
"
        ));
    }
    result
}
