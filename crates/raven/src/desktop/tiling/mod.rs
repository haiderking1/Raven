mod allocation;
mod arrange;
mod configure;
mod geometry;
mod initial;
mod membership;
mod mode;
mod movement;
mod resizing;
pub(crate) use resizing::TileResize;

use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

/// Tile order is independent of Space's focus/stacking order.
#[derive(Default)]
pub(crate) struct Tiling {
    windows: Vec<Window>,
    splits: geometry::adjusted::Splits,
    geometry: Option<Rectangle<i32, Logical>>,
    frames: Vec<Rectangle<i32, Logical>>,
}
