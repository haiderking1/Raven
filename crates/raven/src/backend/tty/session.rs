use smithay::backend::session::{Session, libseat::LibSeatSession};
use smithay::reexports::rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use std::{error::Error, os::fd::OwnedFd, path::Path};

/// Keep the libseat-owned descriptor separate from DRM's shared duplicate.
/// Closing only a duplicate would leave libseat's device registration alive.
pub(super) struct SessionDevice {
    fd: Option<OwnedFd>,
    session: LibSeatSession,
}

impl SessionDevice {
    pub fn open(session: &mut LibSeatSession, path: &Path) -> Result<Self, Box<dyn Error>> {
        let fd = session.open(
            path,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NONBLOCK | OFlags::NOCTTY,
        )?;
        let device = Self {
            fd: Some(fd),
            session: session.clone(),
        };
        // LibSeatSession ignores open flags. Event draining must never block.
        let fd = device.fd.as_ref().expect("new device has a descriptor");
        fcntl_setfl(fd, fcntl_getfl(fd)? | OFlags::NONBLOCK)?;
        Ok(device)
    }

    pub fn duplicate(&self) -> std::io::Result<OwnedFd> {
        self.fd
            .as_ref()
            .expect("device is open until drop")
            .try_clone()
    }
}

impl Drop for SessionDevice {
    fn drop(&mut self) {
        if let Some(fd) = self.fd.take() {
            if let Err(error) = self.session.close(fd) {
                eprintln!("raven: could not release DRM device through libseat: {error}");
            }
        }
    }
}
