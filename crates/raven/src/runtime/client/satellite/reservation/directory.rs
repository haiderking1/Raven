use std::{
    ffi::CStr,
    fs::File,
    io,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::MetadataExt,
    },
    sync::Arc,
};

pub(super) struct Directory {
    pub(super) file: File,
    visible: &'static str,
}

impl Directory {
    pub(super) fn temporary() -> io::Result<Arc<Self>> {
        // SAFETY: the constant is terminated and open returns a new descriptor.
        let fd = unsafe {
            libc::open(
                c"/tmp".as_ptr(),
                libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        let directory = Self {
            file: file(fd)?,
            visible: "/tmp",
        };
        directory.validate()?;
        Ok(Arc::new(directory))
    }

    pub(super) fn sockets(self: &Arc<Self>) -> io::Result<Arc<Self>> {
        // Never chmod or follow an existing directory belonging to another session.
        let created =
            unsafe { libc::mkdirat(self.file.as_raw_fd(), c".X11-unix".as_ptr(), 0o1777) } == 0;
        if !created && io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST) {
            return Err(io::Error::last_os_error());
        }
        let directory = Self {
            file: self.open(c".X11-unix", libc::O_DIRECTORY | libc::O_RDONLY, 0)?,
            visible: "/tmp/.X11-unix",
        };
        directory.validate()?;
        if created {
            // mkdir respects umask, but this shared directory must admit other users.
            // Change only the descriptor opened after our successful mkdir, not an
            // existing directory or a pathname that could now resolve elsewhere.
            if directory.file.metadata()?.uid() != unsafe { libc::geteuid() } {
                return Err(io::Error::other("new X11 directory ownership changed"));
            }
            if unsafe { libc::fchmod(directory.file.as_raw_fd(), 0o1777) } != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(Arc::new(directory))
    }

    fn validate(&self) -> io::Result<()> {
        let metadata = self.file.metadata()?;
        // Sticky directories prevent other users from replacing our entries.
        let uid = unsafe { libc::geteuid() };
        if !metadata.is_dir()
            || ![0, uid].contains(&metadata.uid())
            || metadata.mode() & 0o1000 == 0
        {
            return Err(io::Error::other(
                "X11 reservation requires a trusted sticky directory",
            ));
        }
        Ok(())
    }

    pub(super) fn open(
        &self,
        name: &CStr,
        flags: libc::c_int,
        mode: libc::mode_t,
    ) -> io::Result<File> {
        // SAFETY: name is terminated; the directory fd stays alive through openat.
        file(unsafe {
            libc::openat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                mode,
            )
        })
    }

    pub(super) fn visible(&self) -> bool {
        let Ok(expected) = self.file.metadata() else {
            return false;
        };
        let Ok(current) = std::fs::symlink_metadata(self.visible) else {
            return false;
        };
        expected.dev() == current.dev() && expected.ino() == current.ino()
    }

    pub(super) fn bind_path(&self, name: &str) -> String {
        format!("/proc/self/fd/{}/{name}", self.file.as_raw_fd())
    }
}

fn file(fd: libc::c_int) -> io::Result<File> {
    if fd < 0 {
        Err(io::Error::last_os_error())
    }
    // SAFETY: successful open/openat transferred a fresh descriptor to us.
    else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}
