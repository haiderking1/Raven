use super::geometry::tile_at;
use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

impl State {
    /// Prospective tile for unmapped windows, actual current tile for members.
    pub(crate) fn window_tile_geometry(&self, window: &Window) -> Option<Rectangle<i32, Logical>> {
        if self.window_is_floating(window) {
            return None;
        }
        let index = self.workspaces.index_of(window)?;
        let windows = &self.workspaces.entries[index].tiling.windows;
        let position = windows.iter().position(|candidate| candidate == window);
        let count = windows.len() + usize::from(position.is_none());
        tile_at(
            self.tiling_area()?,
            count,
            position.unwrap_or(windows.len()),
        )
    }
}
