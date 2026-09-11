mod allocation;
mod arrange;
mod configure;
mod geometry;
mod initial;
mod membership;

use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

/// Tile order is independent of Space's focus/stacking order.
#[derive(Default)]
pub(crate) struct Tiling {
    windows: Vec<Window>,
    geometry: Option<Rectangle<i32, Logical>>,
    frames: Vec<Rectangle<i32, Logical>>,
}
