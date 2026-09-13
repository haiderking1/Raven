use std::{
    env,
    ffi::{CString, OsStr, OsString},
    fs,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::Path,
};

fn default_path() -> Result<OsString, String> {
    // execvp uses the system default search path when PATH is absent.
    let length = unsafe { libc::confstr(libc::_CS_PATH, std::ptr::null_mut(), 0) };
    if length == 0 {
        return Err("Cannot determine the system executable search path.".into());
    }
    let mut bytes = vec![0; length];
    let written = unsafe { libc::confstr(libc::_CS_PATH, bytes.as_mut_ptr().cast(), bytes.len()) };
    if written == 0 || written > bytes.len() {
        return Err("Cannot read the system executable search path.".into());
    }
    bytes.truncate(written - 1);
    Ok(OsString::from_vec(bytes))
}

fn executable(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "{} is not a regular executable file.",
            path.display()
        ));
    }
    let path_c = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| "Executable paths cannot contain NUL.".to_owned())?;
    // Check ACLs and effective credentials, not merely permission-mode bits.
    // This probes permission only: it does not execute or read the program.
    if unsafe {
        libc::faccessat(
            libc::AT_FDCWD,
            path_c.as_ptr(),
            libc::X_OK,
            libc::AT_EACCESS,
        )
    } != 0
    {
        return Err(format!(
            "{} is not executable: {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

pub(super) fn check(program: &OsStr) -> Result<(), String> {
    if program.is_empty() {
        return Err("The executable name is empty.".into());
    }
    // Command::spawn inherits Raven's cwd and PATH for terminal/launcher actions.
    // A slash selects a direct path. Empty/relative PATH entries use that same cwd.
    if program.as_bytes().contains(&b'/') {
        return executable(Path::new(program));
    }
    let path = match env::var_os("PATH") {
        Some(path) => path,
        None => default_path()?,
    };
    let mut failure = None;
    for directory in env::split_paths(&path) {
        let candidate = directory.join(program);
        match executable(&candidate) {
            Ok(()) => return Ok(()),
            Err(error) => {
                if candidate.try_exists().unwrap_or(true) {
                    failure = Some(error);
                }
            }
        }
    }
    Err(failure.unwrap_or_else(|| "No executable with that name was found in Raven's PATH.".into()))
}
