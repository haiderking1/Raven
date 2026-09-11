use super::StartupEntry;
use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

pub(super) fn entry(entry: &StartupEntry) -> Result<(), String> {
    let Some(program) = entry.argv.first() else {
        return Err("argv must contain an executable".into());
    };
    if program.is_empty() {
        return Err("executable must not be empty".into());
    }
    for argument in &entry.argv {
        no_nul(argument, "argv")?;
    }
    if let Some(cwd) = &entry.cwd {
        if cwd.as_os_str().is_empty() {
            return Err("cwd must not be empty".into());
        }
        no_nul(cwd.as_os_str(), "cwd")?;
    }
    for (key, value) in &entry.env {
        if key.is_empty() || key.as_bytes().contains(&b'=') {
            return Err("environment names must be nonempty and contain no '='".into());
        }
        no_nul(key, "environment name")?;
        no_nul(value, "environment value")?;
    }
    Ok(())
}

fn no_nul(value: &OsStr, field: &str) -> Result<(), String> {
    if value.as_bytes().contains(&0) {
        Err(format!("{field} must not contain NUL"))
    } else {
        Ok(())
    }
}
