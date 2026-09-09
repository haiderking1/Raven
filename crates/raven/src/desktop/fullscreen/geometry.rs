use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{IsAlive, Logical, Point, Rectangle},
};

impl State {
    /// Full output coordinates, including the output origin, without layer zones.
    pub(crate) fn fullscreen_area(&self) -> Option<Rectangle<i32, Logical>> {
        let output = self.output.as_ref()?;
        output.current_mode()?;
        self.space()
            .output_geometry(output)
            .filter(|area| area.size.w > 0 && area.size.h > 0)
    }

    pub(crate) fn fullscreen_window(&self) -> Option<&Window> {
        let workspace = &self.workspaces.entries[self.workspaces.active];
        let window = workspace.fullscreen.displayed.as_ref()?;
        self.fullscreen_area()?;
        (window.alive() && workspace.space.element_location(window).is_some()).then_some(window)
    }

    /// Allocated layout, not the client's buffer bounds. An exit retains its
    /// fullscreen allocation until the matching tiled state has been committed.
    pub(crate) fn window_layout_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        let index = self.workspaces.index_of(window)?;
        let workspace = &self.workspaces.entries[index];
        workspace.space.element_location(window)?;
        if workspace.fullscreen.displayed.as_ref() == Some(window)
            && self.fullscreen_area().is_some()
            && let Some(area) = workspace
                .fullscreen
                .entries
                .get(window)
                .and_then(|e| e.applied)
        {
            return Some(area);
        }
        self.floating_geometry(window)
            .or_else(|| self.fullscreen_transient_geometry(window))
            .or_else(|| self.window_tile_geometry(window))
    }

    pub(crate) fn fullscreen_manages(&self, window: &Window) -> bool {
        self.workspaces.index_of(window).is_some_and(|index| {
            self.workspaces.entries[index]
                .fullscreen
                .entries
                .get(window)
                .is_some_and(|e| e.intent || e.applied.is_some() || e.transition.is_some())
        })
    }

    pub(crate) fn fullscreen_location(&self, window: &Window) -> Option<Point<i32, Logical>> {
        self.fullscreen_area()?;
        let index = self.workspaces.index_of(window)?;
        let workspace = &self.workspaces.entries[index];
        if workspace.fullscreen.displayed.as_ref() != Some(window) {
            return None;
        }
        let area = workspace.fullscreen.entries.get(window)?.applied?;
        let geometry = window.geometry();
        // Space locations refer to window geometry, not the wl_surface origin.
        // Space itself subtracts geometry.loc when placing the surface tree.
        Some(
            area.loc
                + Point::from((
                    (area.size.w - geometry.size.w) / 2,
                    (area.size.h - geometry.size.h) / 2,
                )),
        )
    }
}
