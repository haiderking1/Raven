//! Observe only the session/group protected by our unreaped leader.
use std::{
    fs, io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

/// SIGKILL has already been sent to this group. Wait on kernel exit notification
/// for remaining members so inherited listening descriptors close before cleanup.
/// This runs only on the satellite worker, never the compositor dispatch thread.
pub(super) fn wait(group: libc::pid_t) -> io::Result<()> {
    let mut members = Vec::new();
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<libc::pid_t>().ok())
        else {
            continue;
        };
        // SAFETY: these calls only inspect process identities. setsid established
        // our child's session/group, and its unreaped PID prevents ID reuse.
        if unsafe { libc::getpgid(pid) != group || libc::getsid(pid) != group } {
            continue;
        }
        let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if fd < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ESRCH) {
                continue;
            }
            return Err(error);
        }
        // SAFETY: pidfd_open returned a fresh descriptor exclusively owned here.
        let fd = unsafe { OwnedFd::from_raw_fd(fd as _) };
        // If the observed process disappeared/recycled during open, do not wait
        // on its unrelated successor. A retained pidfd itself cannot be recycled.
        if unsafe { libc::getpgid(pid) == group && libc::getsid(pid) == group } {
            members.push(fd);
        }
    }
    for member in members {
        let mut event = libc::pollfd {
            fd: member.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        loop {
            // SAFETY: the descriptor remains owned and the pollfd is writable.
            let result = unsafe { libc::poll(&mut event, 1, -1) };
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            if event.revents & (libc::POLLIN | libc::POLLHUP) != 0 {
                break;
            }
            return Err(io::Error::other("owned process exit notification failed"));
        }
    }
    Ok(())
}
