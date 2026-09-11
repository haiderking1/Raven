//! Logical-pixel frame settings, independent of any configuration language.
mod allocation;
mod apply;
mod geometry;
mod settings;

pub use settings::{Appearance, Border, InnerGaps, InvalidAppearance, OuterGaps};
