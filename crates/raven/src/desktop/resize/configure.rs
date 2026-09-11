use crate::state::State;
use smithay::{desktop::Window, reexports::wayland_protocols::xdg::shell::server::xdg_toplevel};

impl State {
    /// Keep one outstanding size request and Smithay's one server_pending target.
    /// The next responding commit sends the newest target, skipping intermediate
    /// sizes without publishing their allocations. Fullscreen mode requests keep
    /// their existing explicit serial/ownership path and may supersede a resize.
    pub(crate) fn send_resize_configure(&mut self, window: &Window) {
        let Some(top) = window.toplevel() else {
            return;
        };
        if !top.has_pending_changes() {
            return;
        }
        let (size, fullscreen) = top.with_pending_state(|pending| {
            (
                pending.size,
                pending.states.contains(xdg_toplevel::State::Fullscreen),
            )
        });
        if let Some(held) = self.resize.held.get_mut(window)
            && let Some(expected) = held.configure
            && !held.applied
            && held.ready.is_none()
            && expected.fullscreen == fullscreen
            && expected.size != size
        {
            held.queued = true;
            return;
        }
        let serial = top.send_pending_configure();
        self.track_resize_configure(window, serial);
    }

    pub(super) fn send_queued_resize(&mut self, window: &Window) {
        if !self.resize.held.get(window).is_some_and(|held| held.queued) {
            return;
        }
        if let Some(held) = self.resize.held.get_mut(window) {
            held.queued = false;
        }
        let Some(top) = window.toplevel() else {
            return;
        };
        let serial = top.send_pending_configure().or_else(|| {
            smithay::wayland::compositor::with_states(top.wl_surface(), |states| {
                let role = states
                    .data_map
                    .get::<smithay::wayland::shell::xdg::XdgToplevelSurfaceData>()?
                    .lock()
                    .unwrap();
                role.pending_configures()
                    .last()
                    .map(|configure| configure.serial)
                    .or(role.configure_serial)
            })
        });
        self.track_resize_configure(window, serial);
    }
}
