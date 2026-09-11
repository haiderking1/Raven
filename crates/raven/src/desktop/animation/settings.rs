use std::{error::Error, fmt, time::Duration};

/// Runtime-ready resize policy. Parsing belongs to the caller, including Lua.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResizeAnimations {
    duration: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidAnimationDuration;

impl fmt::Display for InvalidAnimationDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("resize animation duration must be between 0 and 2000 milliseconds")
    }
}
impl Error for InvalidAnimationDuration {}

impl ResizeAnimations {
    pub const OFF: Self = Self {
        duration: Duration::ZERO,
    };

    pub fn from_millis(milliseconds: u64) -> Result<Self, InvalidAnimationDuration> {
        if milliseconds > 2000 {
            return Err(InvalidAnimationDuration);
        }
        Ok(Self {
            duration: Duration::from_millis(milliseconds),
        })
    }

    pub fn duration(self) -> Duration {
        self.duration
    }
    pub fn enabled(self) -> bool {
        !self.duration.is_zero()
    }
}

impl Default for ResizeAnimations {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(200),
        }
    }
}
