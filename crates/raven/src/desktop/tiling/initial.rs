use super::{configure::configure_tile, geometry::master_stack};
use crate::state::State;
use smithay::desktop::Window;

impl State {
    /// Configure for the assigned workspace without reserving a tile yet.
    pub(crate) fn configure_initial_tile(&self, window: &Window) {
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let count = self.workspaces.entries[index].tiling.windows.len();
        if let Some(area) = self.tiling_area()
            && let Some(tile) = master_stack(area, count + 1).last()
        {
            configure_tile(window, *tile);
        }
    }
}
