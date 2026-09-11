use std::{error::Error, fs, os::unix::fs::MetadataExt, path::Path};

// Capture identities while the reservation is live, never infer ownership from
// an X display number after a timeout. The outer wrapper consumes this on abort.
pub(super) fn record(number: u16, runtime: &Path) -> Result<(), Box<dyn Error>> {
    let socket = fs::symlink_metadata(format!("/tmp/.X11-unix/X{number}"))?;
    let lock_path = format!("/tmp/.X{number}-lock");
    let lock = fs::symlink_metadata(&lock_path)?;
    let owner: u32 = fs::read_to_string(&lock_path)?.trim().parse()?;
    if owner != std::process::id() {
        return Err("reserved X11 lock does not belong to the test process".into());
    }
    let identity = |stat: &fs::Metadata| {
        format!(
            "{} {} {} {} {}",
            stat.dev(),
            stat.ino(),
            stat.uid(),
            stat.ctime(),
            stat.ctime_nsec()
        )
    };
    let pending = runtime.join("x11-ownership.pending");
    fs::write(
        &pending,
        format!(
            "{number} {owner}\n{}\n{}\n",
            identity(&socket),
            identity(&lock)
        ),
    )?;
    fs::rename(pending, runtime.join("x11-ownership"))?;
    Ok(())
}
