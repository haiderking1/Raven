use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Rectangle},
};

pub(super) fn configure_tile(window: &Window, tile: Rectangle<i32, Logical>) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    toplevel.with_pending_state(|state| {
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
}
