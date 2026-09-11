use crate::{backend::tty::TtyBackend, state::State};
use smithay::{
    desktop::Window, reexports::wayland_protocols::xdg::shell::server::xdg_toplevel, utils::Serial,
};

impl State {
    pub(super) fn capture_fullscreen_animation(&mut self, window: &Window, serial: Serial) {
        if !super::firefox::matches(window) || !self.window_is_visible(window) {
            return;
        }
        let Some(top) = window.toplevel() else {
            return;
        };
        let current = top.current_state();
        let pending_fullscreen = self.workspaces.index_of(window).and_then(|index| {
            self.workspaces.entries[index]
                .fullscreen
                .entries
                .get(window)
                .and_then(|entry| entry.transition.as_ref())
                .map(|transition| transition.target.fullscreen)
        });
        if pending_fullscreen == Some(current.states.contains(xdg_toplevel::State::Fullscreen)) {
            return;
        }
        // Record the starting geometry before Firefox moves its desynchronized
        // content surface. This selects live rendering, not an image capture.
        // Root pre-commit updates the serial without resetting that geometry.
        if self.resize_animations().enabled() {
            TtyBackend::capture_resize_animation(self, top.wl_surface(), serial);
        }
    }
}
