use super::{configure::configure_tile, geometry::master_stack};
use crate::state::State;
use smithay::{desktop::Window, reexports::wayland_protocols::xdg::shell::server::xdg_toplevel};

impl State {
    /// Configure for the assigned workspace without reserving a tile yet.
    pub(crate) fn configure_initial_tile(&self, window: &Window) {
        // Smithay retains current mode across null-buffer resets. Reset even
        // without an output, then let the final startup intent override this.
        if let Some(toplevel) = window.toplevel() {
            toplevel.with_pending_state(|state| {
                state.states.unset(xdg_toplevel::State::Fullscreen);
                state.states.unset(xdg_toplevel::State::Maximized);
                state.fullscreen_output = None;
                state.size = None;
                state.bounds = None;
            });
        }
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        if self.configure_floating(window) || self.configure_transient(window) {
            return;
        }
        let count = self.workspaces.entries[index].tiling.windows.len();
        if let Some(area) = self.tiling_area()
            && let Some(tile) = master_stack(area, count + 1).last()
        {
            configure_tile(window, *tile);
        }
    }
}
