use smithay::reexports::rustix::{
    fs::{major, minor},
    process::getuid,
};
use std::{
    error::Error,
    fs,
    os::unix::fs::{FileTypeExt, MetadataExt},
};

pub(super) fn validate() -> Result<(), Box<dyn Error>> {
    let uid = getuid().as_raw();
    if uid == 0 {
        return Err(
            "run Raven as your normal login user, not root; libseat handles device permissions"
                .into(),
        );
    }
    let terminal = fs::metadata("/proc/self/fd/0")?;
    if !terminal.file_type().is_char_device()
        || major(terminal.rdev()) != 4
        || !(1..=63).contains(&minor(terminal.rdev()))
    {
        return Err("direct-TTY mode requires a Linux virtual terminal on stdin; switch to a free VT, log in, and start Raven there".into());
    }
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").ok_or(
        "XDG_RUNTIME_DIR is missing; use a PAM login session that creates your runtime directory",
    )?;
    let metadata = fs::metadata(&runtime)?;
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o777 != 0o700 {
        return Err(
            "XDG_RUNTIME_DIR must be a directory owned by your user with permissions 0700".into(),
        );
    }
    Ok(())
}
