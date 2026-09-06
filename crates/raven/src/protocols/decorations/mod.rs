//! Raven owns decoration policy and currently draws no title bars or borders.
//! Clients that negotiate XDG decorations must submit undecorated content.

use crate::state::State;
use smithay::{
    delegate_xdg_decoration,
    reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
    wayland::{
        compositor::with_states,
        shell::xdg::{ToplevelSurface, XdgToplevelSurfaceData, decoration::XdgDecorationHandler},
    },
};

impl XdgDecorationHandler for State {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        // This flag belongs to the decoration object, which clients can destroy
        // and recreate without destroying the toplevel. Smithay 0.7 retains it.
        with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .expect("XDG toplevel role data")
                .lock()
                .unwrap()
                .initial_decoration_configure_sent = false;
        });
        configure_server_side(&toplevel);
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, _mode: Mode) {
        configure_server_side(&toplevel);
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        configure_server_side(&toplevel);
    }
}

fn configure_server_side(toplevel: &ToplevelSurface) {
    toplevel.with_pending_state(|state| state.decoration_mode = Some(Mode::ServerSide));
    // Initial negotiation travels with the first surface-commit configure,
    // including its tile size. Later set/unset requests require a configure
    // response even if our chosen mode hasn't changed.
    if toplevel.is_initial_configure_sent() {
        toplevel.send_configure();
    }
}

delegate_xdg_decoration!(State);
