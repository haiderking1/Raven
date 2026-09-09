use crate::state::State;
use smithay::wayland::shell::xdg::ToplevelSurface;

impl State {
    pub(super) fn reparent_toplevel(&mut self, surface: ToplevelSurface) {
        let Some(window) = self
            .windows
            .iter()
            .find(|w| w.toplevel() == Some(&surface))
            .cloned()
        else {
            return;
        };
        let Some(index) = self.workspaces.index_of(&window) else {
            return;
        };
        let mapped = self.workspaces.entries[index]
            .space
            .element_location(&window)
            .is_some();
        if !mapped {
            self.prepare_floating(&window);
            // set_parent is immediate, but must not trigger the initial configure.
            if surface.is_initial_configure_sent() {
                if self.fullscreen_manages(&window) {
                    self.configure_fullscreen(&window, true);
                } else {
                    self.configure_initial_tile(&window);
                    surface.send_pending_configure();
                }
            }
        } else {
            // Reparenting changes ancestry/stacking, not the mapped layout mode.
            self.restack_floating(index);
            self.retile_workspace(index);
            self.reconcile_transient_visibility();
        }
    }
}
