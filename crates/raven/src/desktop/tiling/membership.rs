use crate::state::State;
use smithay::{desktop::Window, utils::IsAlive};

impl State {
    pub(crate) fn map_tiled_window(&mut self, index: usize, window: Window) {
        let origin = self.tiling_area().map(|area| area.loc).unwrap_or_default();
        let workspace = &mut self.workspaces.entries[index];
        if !workspace.tiling.windows.contains(&window) {
            workspace.tiling.windows.push(window.clone());
        }
        workspace.space.map_element(window, origin, false);
        self.retile_workspace(index);
    }

    pub(crate) fn refresh_workspace_tiling(&mut self, index: usize) {
        let area = self.tiling_area();
        let workspace = &mut self.workspaces.entries[index];
        let before = workspace.tiling.windows.len();
        workspace
            .tiling
            .windows
            .retain(|window| window.alive() && workspace.space.element_location(window).is_some());
        if before != workspace.tiling.windows.len() || workspace.tiling.geometry != area {
            self.retile_workspace(index);
        }
    }

    pub(crate) fn refresh_tiling(&mut self) {
        for index in 0..super::super::workspaces::COUNT {
            self.refresh_workspace_tiling(index);
        }
    }
}
