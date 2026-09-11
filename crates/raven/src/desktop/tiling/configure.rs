use crate::desktop::appearance::configure_client_decorations;
use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Rectangle},
};

impl State {
    /// Only callers without an in-flight fullscreen transition may use this.
    pub(crate) fn configure_transient(&self, window: &Window) -> bool {
        if self.window_is_floating(window) {
            return self.configure_floating(window);
        }
        let Some(area) = self.refresh_fullscreen_transient_geometry(window) else {
            return false;
        };
        let Some(toplevel) = window.toplevel() else {
            return false;
        };
        toplevel.with_pending_state(|state| {
            state.size = Some(area.size);
            state.bounds = Some(area.size);
            state.fullscreen_output = None;
            for flag in [
                xdg_toplevel::State::Fullscreen,
                xdg_toplevel::State::TiledLeft,
                xdg_toplevel::State::TiledRight,
                xdg_toplevel::State::TiledTop,
                xdg_toplevel::State::TiledBottom,
            ] {
                state.states.unset(flag);
            }
        });
        configure_client_decorations(window);
        true
    }
}

pub(super) fn configure_tile(window: &Window, tile: Rectangle<i32, Logical>) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    toplevel.with_pending_state(|state| {
        state.states.unset(xdg_toplevel::State::Fullscreen);
        state.fullscreen_output = None;
        state.size = Some(tile.size);
        state.bounds = Some(tile.size);
        for edge in [
            xdg_toplevel::State::TiledLeft,
            xdg_toplevel::State::TiledRight,
            xdg_toplevel::State::TiledTop,
            xdg_toplevel::State::TiledBottom,
        ] {
            state.states.set(edge);
        }
    });
    configure_client_decorations(window);
}
