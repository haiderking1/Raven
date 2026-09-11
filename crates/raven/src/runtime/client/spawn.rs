use super::super::signals;
use std::{
    collections::BTreeMap,
    error::Error,
    ffi::{OsStr, OsString},
    os::unix::process::CommandExt,
    path::Path,
    process::{Child, Command},
};

pub(super) fn client(
    args: &[OsString],
    cwd: Option<&Path>,
    env: &BTreeMap<OsString, OsString>,
    socket: &OsStr,
    display: Option<&OsStr>,
) -> Result<Option<Child>, Box<dyn Error>> {
    let Some(program) = args.first() else {
        return Ok(None);
    };
    let mut command = Command::new(program);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    // Apply entry overrides first; Raven owns the child's display routing.
    command
        .args(&args[1..])
        .envs(env)
        .env("WAYLAND_DISPLAY", socket)
        .env("XDG_SESSION_TYPE", "wayland")
        .env("XDG_CURRENT_DESKTOP", "Raven")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_SOCKET");
    // The socket name resolves inside Raven's runtime directory, not an override.
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        command.env("XDG_RUNTIME_DIR", runtime_dir);
    } else {
        command.env_remove("XDG_RUNTIME_DIR");
    }
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
