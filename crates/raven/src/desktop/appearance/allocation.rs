use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

impl State {
    /// Allocated frame in output logical coordinates, not client buffer bounds.
    /// Only the displayed fullscreen owner has no border, including during exit.
    pub fn window_frame_geometry(&self, window: &Window) -> Option<Rectangle<i32, Logical>> {
        let index = self.workspaces.index_of(window)?;
        self.workspaces.entries[index]
            .space
            .element_location(window)?;
        self.resize_displayed_frame(window)
            .or_else(|| self.window_target_frame_geometry(window))
    }

    /// Configure allocation, never the frozen displayed frame.
    pub(crate) fn window_target_frame_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        self.applied_fullscreen_allocation(window)
            .or_else(|| self.floating_frame_geometry(window))
            .or_else(|| self.fullscreen_transient_frame_geometry(window))
            .or_else(|| self.window_tile_frame_geometry(window))
    }

    /// Client allocation excludes borders. Rendering and root-surface input must
    /// clip to this rectangle even if the client ignores its configured size.
    pub fn window_client_geometry(&self, window: &Window) -> Option<Rectangle<i32, Logical>> {
        let index = self.workspaces.index_of(window)?;
        self.workspaces.entries[index]
            .space
            .element_location(window)?;
        if let Some(area) = self.resize_displayed_client(window) {
            return Some(area);
        }
        self.window_target_client_geometry(window)
    }

    pub(crate) fn window_target_client_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        if let Some(area) = self.applied_fullscreen_allocation(window) {
            return Some(area);
        }
        self.window_target_frame_geometry(window)
            .map(|frame| self.appearance.client_rect(frame))
    }

    /// Compatibility name for client allocation, never the decorated frame.
    pub(crate) fn window_layout_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        self.window_client_geometry(window)
    }
}
