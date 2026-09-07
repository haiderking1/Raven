use super::COUNT;
use crate::state::State;

impl State {
    pub(crate) fn move_focused_to_workspace(&mut self, index: usize) {
        let source = self.workspaces.active;
        if index >= COUNT || index == source || !self.prepare_workspace_change() {
            return;
        }
        let Some(window) = self.focused_window() else {
            return;
        };
        if let Some(toplevel) = window.toplevel() {
            self.dismiss_window_popups(toplevel.wl_surface());
            window.set_activated(false);
            toplevel.send_pending_configure();
        }
        self.request_redraw();
        self.space_mut().unmap_elem(&window);
        self.workspaces.assign(window.clone(), index);
        self.map_tiled_window(index, window);
        self.refresh_workspace_tiling(source);
        self.restore_focus();
        self.refresh_tiling_pointer();
    }
}
