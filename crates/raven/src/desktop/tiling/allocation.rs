use super::geometry::spaced_tile_at;
use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

impl State {
    /// Configure target, excluding the compositor frame. Display consumers must
    /// use window_client_geometry, which retains the transaction allocation.
    pub(crate) fn window_tile_geometry(&self, window: &Window) -> Option<Rectangle<i32, Logical>> {
        self.window_tile_frame_geometry(window)
            .map(|frame| self.appearance.client_rect(frame))
    }

    /// Target frame for mapped members; prospective frame for opening clients.
    /// This deliberately bypasses the displayed resize-transaction allocation.
    pub(crate) fn window_tile_frame_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        if self.window_is_floating(window) {
            return None;
        }
        let index = self.workspaces.index_of(window)?;
        let windows = &self.workspaces.entries[index].tiling.windows;
        if self.workspaces.entries[index].tiling.geometry.is_none() && windows.contains(window) {
            return None;
        }
        let position = windows.iter().position(|candidate| candidate == window);
        let count = windows.len() + usize::from(position.is_none());
        if let Some(position) = position {
            return self.workspaces.entries[index]
                .tiling
                .frames
                .get(position)
                .copied();
        }
        spaced_tile_at(
            self.tiling_area()?,
            count,
            position.unwrap_or(windows.len()),
            self.appearance.inner,
        )
    }
}
