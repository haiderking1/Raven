//! Only the satellite worker waits for these children. Keep the leader unreaped
//! until its process group is killed, so a recycled PID cannot target a new session.
mod members;

use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
    process::{Child, ExitStatus},
};

pub(crate) struct Group {
    child: Option<Child>,
    pidfd: Option<OwnedFd>,
}

impl Group {
    pub(crate) fn new(child: Child) -> io::Result<Self> {
        let mut group = Self {
            child: Some(child),
            pidfd: None,
        };
        // SAFETY: the live, unreaped child owns this PID. The returned descriptor
        // is new, close-on-exec, and becomes exclusively owned by this group.
        let fd =
            unsafe { libc::syscall(libc::SYS_pidfd_open, group.child.as_ref().unwrap().id(), 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        group.pidfd = Some(unsafe { OwnedFd::from_raw_fd(fd as RawFd) });
        Ok(group)
    }

    pub(crate) fn poll_fd(&self) -> RawFd {
        self.pidfd
            .as_ref()
            .expect("live child descriptor")
            .as_raw_fd()
    }

    pub(crate) fn exited(&self) -> io::Result<bool> {
        let Some(child) = &self.child else {
            return Ok(true);
        };
        let mut info = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
        // SAFETY: waitid writes a valid siginfo_t and WNOWAIT retains ownership of
        // the child's PID even when it has exited. No wait(-1) is used anywhere.
        loop {
            let result = unsafe {
                libc::waitid(
                    libc::P_PID,
                    child.id(),
                    info.as_mut_ptr(),
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            if result == 0 {
                break;
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
        Ok(unsafe { info.assume_init().si_pid() } != 0)
    }

    pub(crate) fn finish(mut self) -> io::Result<ExitStatus> {
        self.terminate()
    }

    fn terminate(&mut self) -> io::Result<ExitStatus> {
        self.kill_group()?;
        // The leader can exit before its Xwayland child closes inherited sockets.
        // Keep its PID reserved while observing the rest of the owned group.
        let members = members::wait(self.child.as_ref().expect("owned child").id() as _);
        let result = self.child.as_mut().expect("owned child").wait();
        if result.is_ok() {
            self.child.take();
        }
        members?;
        result
    }

    fn kill_group(&self) -> io::Result<()> {
        // If ownership was unexpectedly lost, never signal a possibly recycled ID.
        self.exited()?;
        if let Some(child) = &self.child {
            // setsid in pre_exec made this child the group leader. Xwayland inherits
            // that group, so it cannot keep the listening sockets alive after exit.
            let result = unsafe { libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL) };
            if result != 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        if self.child.is_none() {
            return;
        }
        if let Err(error) = self.terminate() {
            eprintln!("raven: cannot clean up satellite process group: {error}");
        }
    }
}
