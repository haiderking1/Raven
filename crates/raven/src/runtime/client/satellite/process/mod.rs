mod group;
pub(super) use group::Group;

use crate::runtime::signals;
use std::{
    ffi::OsStr,
    io,
    os::{fd::RawFd, unix::process::CommandExt},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn command(binary: &OsStr) -> Command {
    let mut command = Command::new(binary);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .env_remove("DISPLAY")
        .env_remove("XAUTHORITY")
        .env_remove("WAYLAND_SOCKET")
        .env_remove("NOTIFY_SOCKET")
        .env_remove("LISTEN_PID")
        .env_remove("LISTEN_FDS")
        .env_remove("LISTEN_FDNAMES");
    // SAFETY: only async-signal-safe syscalls run between fork and exec. A separate
    // session prevents shutdown signals from reaching Raven or foreign clients.
    unsafe {
        command.pre_exec(|| {
            signals::unblock_in_child()?;
            if libc::setsid() < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    command
}

/// Run before backend installation, on the worker, with no route to a Wayland
/// session and no inherited listenfds. Even the probe gets owned-group cleanup.
pub(super) fn supported(binary: &OsStr, display: &str) -> io::Result<()> {
    let child = command(binary)
        .args([display, "--test-listenfd-support"])
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("XDG_RUNTIME_DIR")
        .stderr(Stdio::null())
        .spawn()?;
    let group = Group::new(child)?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if group.exited()? {
            let status = group.finish()?;
            return if status.success() {
                Ok(())
            } else {
                Err(io::Error::other(format!(
                    "satellite lacks listenfd support: {status}"
                )))
            };
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "satellite capability check exceeded three seconds",
            ));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        let mut notification = libc::pollfd {
            fd: group.poll_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: notification owns no descriptor; the live Group retains it.
        let result = unsafe {
            libc::poll(
                &mut notification,
                1,
                remaining.as_millis().clamp(1, 3000) as i32,
            )
        };
        if result < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            return Err(io::Error::last_os_error());
        }
    }
}

pub(super) fn launch(
    binary: &OsStr,
    socket: &OsStr,
    display: &str,
    fds: [RawFd; 2],
) -> io::Result<Group> {
    let mut command = command(binary);
    command
        .arg(display)
        .env("WAYLAND_DISPLAY", socket)
        .env("XDG_SESSION_TYPE", "wayland")
        .env("XDG_CURRENT_DESKTOP", "Raven");
    for fd in fds {
        command.arg("-listenfd").arg(fd.to_string());
    }
    // SAFETY: the worker retains these fds throughout spawn. CLOEXEC changes only
    // in the forked child, leaving Raven and unrelated managed children unchanged.
    unsafe {
        command.pre_exec(move || {
            for fd in fds {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
    command.spawn().and_then(Group::new)
}
