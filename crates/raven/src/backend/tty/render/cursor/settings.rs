use std::{env, io};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub theme: String,
    pub size: u32,
}

impl Settings {
    pub fn from_env() -> io::Result<Self> {
        let read = |name| match env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, error)),
        };
        let theme = read("XCURSOR_THEME")?.unwrap_or_else(|| "default".into());
        if theme.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "XCURSOR_THEME must not be empty",
            ));
        }
        let size = match read("XCURSOR_SIZE")? {
            None => 24,
            Some(value) => value
                .parse::<u32>()
                .ok()
                .filter(|size| *size > 0 && *size <= i32::MAX as u32)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "XCURSOR_SIZE must be a positive logical-pixel integer fitting i32",
                    )
                })?,
        };
        Ok(Self { theme, size })
    }
}
