use super::super::signals;
use std::{
    error::Error,
    ffi::{OsStr, OsString},
    os::unix::process::CommandExt,
    process::{Child, Command},
};

pub(super) fn client(
    args: &[OsString],
    socket: &OsStr,
    display: Option<&OsStr>,
) -> Result<Option<Child>, Box<dyn Error>> {
    let Some(program) = args.first() else {
        return Ok(None);
    };
    let mut command = Command::new(program);
    command
        .args(&args[1..])
        .env("WAYLAND_DISPLAY", socket)
        .env("XDG_SESSION_TYPE", "wayland")
        .env("XDG_CURRENT_DESKTOP", "Raven")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_SOCKET");
    if let Some(display) = display {
        command.env("DISPLAY", display).env_remove("XAUTHORITY");
    }
    // SAFETY: the callback only invokes async-signal-safe signal syscalls.
    unsafe {
        command.pre_exec(signals::unblock_in_child);
    }
    command
        .spawn()
        .map(Some)
        .map_err(|error| format!("cannot launch {}: {error}", program.to_string_lossy()).into())
}
