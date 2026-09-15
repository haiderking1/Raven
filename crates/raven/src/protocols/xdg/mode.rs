use crate::state::State;
use smithay::{
    utils::Serial,
    wayland::{
        compositor::with_states,
        shell::xdg::{ToplevelSurface, XdgToplevelSurfaceData},
    },
};
fn latest(surface: &ToplevelSurface) -> Option<Serial> {
    with_states(surface.wl_surface(), |states| {
        states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .and_then(|data| {
                data.lock()
                    .unwrap()
                    .pending_configures()
                    .last()
                    .map(|configure| configure.serial)
            })
    })
}
/// Answer unchanged/refused requests once, but do not duplicate a configure
/// already issued by the resize transaction or send one before initial commit.
pub(super) fn maximize(state: &mut State, surface: &ToplevelSurface, maximized: bool) {
    let before = latest(surface);
    if let Some(window) = state
        .windows
        .iter()
        .find(|window| window.toplevel() == Some(surface))
        .cloned()
    {
        state.set_window_maximized(&window, maximized);
    }
    if surface.is_initial_configure_sent() && latest(surface) == before {
        surface.send_configure();
    }
}
