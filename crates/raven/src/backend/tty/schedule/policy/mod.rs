use std::{env, ffi::OsString, fmt, str::FromStr};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Policy {
    /// Render whenever the single KMS slot is free. No successor rendering.
    Immediate,
    /// Always permit one successor near the predicted pending pageflip.
    Deadline,
    /// Use deadline overlap only when measured render pressure warrants it.
    #[default]
    Adaptive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyError {
    value: OsString,
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid RAVEN_FRAME_PIPELINE value {:?}; expected adaptive, immediate or deadline",
            self.value
        )
    }
}

impl std::error::Error for PolicyError {}

impl Policy {
    pub fn from_env() -> Result<Self, PolicyError> {
        Self::from_value(env::var_os("RAVEN_FRAME_PIPELINE"))
    }

    fn from_value(value: Option<OsString>) -> Result<Self, PolicyError> {
        match value {
            None => Ok(Self::default()),
            Some(value) => match value.to_str() {
                Some("immediate") => Ok(Self::Immediate),
                Some("deadline") => Ok(Self::Deadline),
                Some("adaptive") => Ok(Self::Adaptive),
                _ => Err(PolicyError { value }),
            },
        }
    }
}

impl FromStr for Policy {
    type Err = PolicyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_value(Some(value.into()))
    }
}

#[cfg(test)]
mod tests;
