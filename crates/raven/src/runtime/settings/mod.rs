//! Validated runtime values shared by Lua and programmatic configuration.
mod defaults;
mod validation;
mod workspaces;
pub use crate::backend::tty::CursorSettings;
pub use crate::input::{Action, Bindings};
pub use workspaces::WorkspaceSettings;

pub use super::startup::{StartupEntry, StartupPlan};
pub use crate::desktop::animation::{InvalidAnimationDuration, ResizeAnimations};
pub use crate::desktop::appearance::{Appearance, Border, InnerGaps, InvalidAppearance, OuterGaps};

/// Values are validated before backend acquisition; constructing settings starts nothing.
#[derive(Clone, Debug)]
pub struct Settings {
    pub appearance: Appearance,
    pub resize_animations: ResizeAnimations,
    pub startup: StartupPlan,
    pub bindings: Bindings,
    pub terminal: Vec<std::ffi::OsString>,
    pub launcher: Vec<std::ffi::OsString>,
    pub cursor: CursorSettings,
    pub workspaces: WorkspaceSettings,
}
