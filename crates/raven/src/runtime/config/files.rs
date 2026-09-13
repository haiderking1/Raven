use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

pub(super) const LIMIT: u64 = 1024 * 1024;

pub(super) fn path() -> io::Result<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        Some(value) if PathBuf::from(&value).is_absolute() => PathBuf::from(value),
        _ => PathBuf::from(
            std::env::var_os("HOME").ok_or_else(|| io::Error::other("HOME is not set"))?,
        )
        .join(".config"),
    };
    Ok(base.join("raven/raven.lua"))
}

pub(super) fn ensure(path: &std::path::Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| io::Error::other("configuration path has no parent"))?,
    )?;
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => file.write_all(include_bytes!("defaults/raven.lua")),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn read(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .and_then(|file| {
            if !file.metadata()?.is_file() {
                return Err(io::Error::other("configuration must be a regular file"));
            }
            file.take(LIMIT + 1).read_to_end(&mut bytes)
        })
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() as u64 > LIMIT {
        return Err(format!("{}: configuration exceeds 1 MiB", path.display()));
    }
    Ok(bytes)
}
