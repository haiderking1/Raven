use crate::state::State;
use smithay::{desktop::Window, utils::IsAlive};

impl State {
    pub(crate) fn window_is_tiled(&self, window: &Window) -> bool {
        self.workspaces.index_of(window).is_some_and(|index| {
            self.workspaces.entries[index]
                .tiling
                .windows
                .contains(window)
        })
    }

    /// Swap layout membership, not Space's focus/stacking order. The existing
    /// resize transaction applies both clients' new allocations together.
    pub(crate) fn swap_dragged_tiles(&mut self, source: &Window, target: &Window) {
        let index = self.workspaces.active;
        if source == target
            || !source.alive()
            || !target.alive()
            || self.workspaces.index_of(source) != Some(index)
            || self.workspaces.index_of(target) != Some(index)
            || !self.window_is_visible(source)
            || !self.window_is_visible(target)
            || self.fullscreen_manages(source)
            || self.fullscreen_manages(target)
        {
            return;
        }
        let members = &self.workspaces.entries[index].tiling.windows;
        let (Some(a), Some(b)) = (
            members.iter().position(|window| window == source),
            members.iter().position(|window| window == target),
        ) else {
            return;
        };
        // Capture the old per-window allocations before changing their indices.
        self.begin_resize_batch(index);
        self.workspaces.entries[index].tiling.windows.swap(a, b);
        self.retile_workspace(index);
        self.end_resize_batch();
        self.activate_window(Some(source.clone()));
    }
}
