//! Runtime values shared by the built-in defaults and a future Lua loader.

pub use super::startup::{StartupEntry, StartupPlan};
pub use crate::desktop::animation::{InvalidAnimationDuration, ResizeAnimations};
pub use crate::desktop::appearance::{Appearance, Border, InnerGaps, InvalidAppearance, OuterGaps};

/// Values are validated before backend acquisition; constructing settings starts nothing.
#[derive(Clone, Debug, Default)]
pub struct Settings {
    pub appearance: Appearance,
    pub resize_animations: ResizeAnimations,
    pub startup: StartupPlan,
}
