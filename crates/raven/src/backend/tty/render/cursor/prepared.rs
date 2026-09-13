use super::{frames::Animation, settings::Settings, theme};
use smithay::input::pointer::CursorIcon;
use std::io;
use xcursor::CursorTheme;

/// CPU-only preparation can run on the configuration worker. No GL imports occur.
pub(crate) struct PreparedCursor {
    pub(super) settings: Settings,
    pub(super) theme: CursorTheme,
    pub(super) fallback: Animation,
}
impl PreparedCursor {
    pub(crate) fn load(settings: Settings) -> io::Result<Self> {
        if settings.theme.trim().is_empty() || settings.size == 0 || settings.size > i32::MAX as u32
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cursor requires a theme and a positive size fitting i32",
            ));
        }
        let theme = CursorTheme::load(&settings.theme);
        let fallback = theme::load(&theme, CursorIcon::Default, settings.size, settings.size)
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("cursor theme {:?}: {error}", settings.theme),
                )
            })?;
        Ok(Self {
            settings,
            theme,
            fallback,
        })
    }
}
