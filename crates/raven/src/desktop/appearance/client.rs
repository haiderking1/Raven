use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

pub(crate) fn is_firefox(window: &Window) -> bool {
    let Some(top) = window.toplevel() else {
        return false;
    };
    with_states(top.wl_surface(), |states| {
        let Some(role) = states.data_map.get::<XdgToplevelSurfaceData>() else {
            return false;
        };
        let role = role.lock().unwrap();
        role.app_id.as_deref().is_some_and(|id| {
            matches!(
                id.to_ascii_lowercase().as_str(),
                "firefox" | "firefox-esr" | "org.mozilla.firefox" | "org.mozilla.firefox_esr"
            )
        })
    })
}

/// GTK can restore CSD shadow margins despite all four tiled flags. Keep the
/// maximized hint stable for Firefox, including floating and fullscreen exits.
/// This affects client decoration policy, not Raven's allocation or layout mode.
pub(crate) fn configure_client_decorations(window: &Window) {
    let maximized_hint = is_firefox(window);
    let Some(top) = window.toplevel() else {
        return;
    };
    // Identify the client before taking Smithay's pending-state role lock.
    top.with_pending_state(|state| {
        if maximized_hint {
            state.states.set(xdg_toplevel::State::Maximized);
        } else {
            state.states.unset(xdg_toplevel::State::Maximized);
        }
    });
}
