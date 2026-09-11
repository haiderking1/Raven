use super::configure::configure_tile;
use crate::desktop::appearance::configure_client_decorations;
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
                state.fullscreen_output = None;
                state.size = None;
                state.bounds = None;
            });
            configure_client_decorations(window);
        }
        if self.configure_floating(window) || self.configure_transient(window) {
            return;
        }
        if let Some(tile) = self.window_tile_geometry(window) {
            configure_tile(window, tile);
        }
    }
}
