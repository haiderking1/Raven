use std::{
    ffi::CString,
    fs::OpenOptions,
    io::{self, Write},
    os::unix::{ffi::OsStrExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
};
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
pub(super) fn publish(path: &Path, png: &[u8]) -> io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::other("missing screenshot filename"))?;
    let temporary = path.with_file_name(format!(
        ".raven-{}-{}.tmp",
        std::process::id(),
        name.to_string_lossy()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    let temporary = Temporary(temporary);
    file.write_all(png)?;
    file.sync_all()?;
    drop(file);
    let from = CString::new(temporary.0.as_os_str().as_bytes())?;
    let to = CString::new(path.as_os_str().as_bytes())?;
    // Publish only complete PNGs. Do not replace an existing file or symlink.
    if unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn screenshot_publication_never_replaces_an_existing_file() {
        let root = std::env::temp_dir().join(format!(
            "raven-png-publish-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let target = root.join("image.png");
        std::fs::write(&target, b"original").unwrap();
        assert_eq!(
            super::publish(&target, b"replacement").unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );
        assert_eq!(std::fs::read(&target).unwrap(), b"original");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
