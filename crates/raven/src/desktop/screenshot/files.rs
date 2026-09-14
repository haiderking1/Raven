mod publish;
use std::{
    fs::DirBuilder,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub(super) fn destination() -> Result<PathBuf, String> {
    let pictures = super::native::pictures()
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|h| !h.is_empty())
                .map(|h| PathBuf::from(h).join("Pictures"))
        })
        .ok_or("cannot find the Pictures directory")?;
    if !pictures.is_absolute() {
        return Err("Pictures directory must be absolute".into());
    }
    Ok(pictures.join("Screenshots"))
}
pub(super) fn save(directory: &Path, png: &[u8]) -> Result<PathBuf, String> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)
        .map_err(|e| format!("cannot create {}: {e}", directory.display()))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let seconds = now.as_secs() as libc::time_t;
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    let mut date = [0u8; 64];
    unsafe {
        if libc::localtime_r(&seconds, &mut tm).is_null()
            || libc::strftime(
                date.as_mut_ptr().cast(),
                date.len(),
                c"%Y-%m-%d %H-%M-%S".as_ptr(),
                &tm,
            ) == 0
        {
            return Err("cannot format screenshot time".into());
        }
    }
    let date = unsafe { std::ffi::CStr::from_ptr(date.as_ptr().cast()) }.to_string_lossy();
    for sequence in 0..16 {
        let path = directory.join(format!(
            "Screenshot {date}-{:09}-{sequence}.png",
            now.subsec_nanos()
        ));
        match publish::publish(&path, png) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("cannot save {}: {e}", path.display())),
        }
        return Ok(path);
    }
    Err("cannot allocate a unique screenshot filename".into())
}
