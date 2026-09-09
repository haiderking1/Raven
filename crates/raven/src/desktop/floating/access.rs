use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

impl State {
    /// Floating mode includes temporarily fullscreen and pre-map windows.
    pub fn window_is_floating(&self, window: &Window) -> bool {
        self.workspaces.index_of(window).is_some_and(|index| {
            self.workspaces.entries[index]
                .floating
                .entries
                .contains_key(window)
        })
    }

    /// Mapped floaters on a zero-based workspace, including fullscreen floaters.
    #[cfg(test)]
    pub fn workspace_floating_count(&self, index: usize) -> usize {
        self.workspaces.entries.get(index).map_or(0, |workspace| {
            workspace
                .floating
                .entries
                .keys()
                .filter(|window| workspace.space.element_location(window).is_some())
                .count()
        })
    }

    /// Normal floating allocation, retained while fullscreen; None before placement.
    pub fn floating_geometry(&self, window: &Window) -> Option<Rectangle<i32, Logical>> {
        let index = self.workspaces.index_of(window)?;
        self.workspaces.entries[index]
            .floating
            .entries
            .get(window)?
            .geometry
    }
}
