//! Logical-pixel frame settings, independent of any configuration language.
mod allocation;
mod client;
pub(crate) use client::{configure_client_decorations, is_firefox};
mod apply;
mod content;
mod geometry;
mod settings;

pub use settings::{Appearance, Border, InnerGaps, InvalidAppearance, OuterGaps};
