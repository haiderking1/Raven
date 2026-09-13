use crate::state::State;
use smithay::desktop::Window;

impl State {
    /// Call inside a resize batch, before publishing the new floating mode.
    pub(crate) fn detach_floating_tile(&mut self, window: &Window) -> Option<usize> {
        let index = self.workspaces.index_of(window)?;
        let windows = &mut self.workspaces.entries[index].tiling.windows;
        let slot = windows.iter().position(|candidate| candidate == window)?;
        windows.remove(slot);
        Some(slot)
    }

    pub(crate) fn restore_floating_tile(&mut self, window: &Window, slot: Option<usize>) {
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let windows = &mut self.workspaces.entries[index].tiling.windows;
        if !windows.contains(window) {
            let slot = slot.unwrap_or(windows.len()).min(windows.len());
            windows.insert(slot, window.clone());
        }
    }
}
