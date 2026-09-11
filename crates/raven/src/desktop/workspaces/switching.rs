use super::COUNT;
use crate::state::State;

impl State {
    pub(crate) fn switch_workspace(&mut self, index: usize) {
        self.cancel_workspace_activation();
        if index >= COUNT || index == self.workspaces.active || !self.prepare_workspace_change() {
            return;
        }
        self.cancel_resize_transactions();
        self.cancel_resize_visuals();
        let leaving: Vec<_> = self.space().elements().cloned().collect();
        for window in leaving {
            if let Some(toplevel) = window.toplevel() {
                self.dismiss_window_popups(toplevel.wl_surface());
                window.set_activated(false);
                toplevel.send_pending_configure();
            }
        }
        let output = self.output.clone();
        let location = output
            .as_ref()
            .and_then(|output| self.space().output_geometry(output))
            .map(|geometry| geometry.loc)
            .unwrap_or_default();
        if let Some(output) = &output {
            self.space_mut().unmap_output(output);
        }
        self.workspaces.active = index;
        self.request_redraw();
        if let Some(output) = &output {
            self.space_mut().map_output(output, location);
        }
        self.refresh_workspace_tiling(index);
        self.refresh_fullscreen();
        let workspace = &self.workspaces.entries[index];
        let focus = workspace
            .focused
            .as_ref()
            .filter(|window| self.window_is_visible(window))
            .cloned()
            .or_else(|| self.fullscreen_window().cloned())
            .or_else(|| {
                workspace
                    .space
                    .elements()
                    .rev()
                    .find(|w| self.window_is_visible(w))
                    .cloned()
            });
        self.activate_window(focus);
        // Also send leave when switching to an empty workspace or without output.
        self.refresh_tiling_pointer();
    }
}
