use smithay::{
    desktop::Window,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

pub(super) use crate::desktop::appearance::is_firefox as matches;

impl crate::state::State {
    pub(crate) fn firefox_live_fullscreen_animation(&self, window: &Window) -> bool {
        if !matches(window) {
            return false;
        }
        let Some(top) = window.toplevel() else {
            return false;
        };
        with_states(top.wl_surface(), |states| {
            use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State;
            let Some(role) = states.data_map.get::<XdgToplevelSurfaceData>() else {
                return false;
            };
            let role = role.lock().unwrap();
            role.current_server_state()
                .states
                .contains(State::Fullscreen)
                != role.current.states.contains(State::Fullscreen)
        })
    }
}
