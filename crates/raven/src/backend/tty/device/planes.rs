use smithay::backend::drm::compositor::FrameFlags;
use std::{env, io};

/// Client primary scanout and compositor-owned cursor BOs are separate choices.
#[derive(Clone, Copy, Debug)]
pub(in crate::backend::tty) struct PlanePolicy {
    pub primary: bool,
    pub cursor: bool,
    suspended: bool,
}

impl PlanePolicy {
    pub fn from_env() -> io::Result<Self> {
        Ok(Self {
            primary: enabled("RAVEN_PRIMARY_SCANOUT", true)?,
            // Smithay 0.7 has unchecked cursor BO mapping and incomplete fast-copy
            // padding clearing. Keep this experiment opt-in until those are fixed.
            cursor: enabled("RAVEN_HARDWARE_CURSOR", false)?,
            suspended: false,
        })
    }

    /// A driver rejection disables optional planes until the next VT activation.
    pub fn suspend(&mut self) {
        self.suspended = true;
    }
    pub fn resume(&mut self) {
        self.suspended = false;
    }

    pub fn flags(self) -> FrameFlags {
        if self.suspended {
            return FrameFlags::empty();
        }
        let mut flags = FrameFlags::empty();
        if self.primary {
            flags |= FrameFlags::ALLOW_PRIMARY_PLANE_SCANOUT;
        }
        if self.cursor {
            flags |= FrameFlags::ALLOW_CURSOR_PLANE_SCANOUT;
        }
        // No ANY-format primary, overlays, or skipped cursor-only updates.
        flags
    }
}

fn enabled(name: &str, default: bool) -> io::Result<bool> {
    match env::var(name) {
        Ok(value) => match value.as_str() {
            "1" | "true" => Ok(true),
            "0" | "false" => Ok(false),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{name} must be 0, 1, false, or true"),
            )),
        },
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
    }
}
