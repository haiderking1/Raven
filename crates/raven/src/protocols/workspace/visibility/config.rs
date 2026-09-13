use crate::desktop::workspaces::COUNT;
use std::{env, io};

/// Workspace numbers are user-facing and one-based. An empty value means no pins.
pub(in crate::protocols::workspace) fn persistent_from_env() -> io::Result<[bool; COUNT]> {
    match env::var("RAVEN_WORKSPACE_PERSISTENT") {
        Ok(value) => parse(&value),
        Err(env::VarError::NotPresent) => Ok([false; COUNT]),
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
    }
}

fn parse(value: &str) -> io::Result<[bool; COUNT]> {
    let mut persistent = [false; COUNT];
    if value.trim().is_empty() {
        return Ok(persistent);
    }
    for entry in value.split(',') {
        let number = entry.trim().parse::<usize>().ok()
            .filter(|number| (1..=COUNT).contains(number))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                format!("RAVEN_WORKSPACE_PERSISTENT: expected comma-separated workspace numbers 1..={COUNT}, got {entry:?}")))?;
        persistent[number - 1] = true;
    }
    Ok(persistent)
}
