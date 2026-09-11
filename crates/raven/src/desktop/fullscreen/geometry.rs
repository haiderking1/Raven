use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{IsAlive, Logical, Point, Rectangle},
};

impl State {
    pub(crate) fn applied_fullscreen_allocation(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        let index = self.workspaces.index_of(window)?;
        let workspace = &self.workspaces.entries[index];
        if workspace.fullscreen.displayed.as_ref() != Some(window) {
            return None;
        }
        self.fullscreen_area()?;
        workspace.fullscreen.entries.get(window)?.applied
    }

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
