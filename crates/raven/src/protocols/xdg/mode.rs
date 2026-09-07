use smithay::wayland::shell::xdg::ToplevelSurface;

/// Raven currently keeps all toplevels tiled and advertises no optional WM modes.
/// A mode request must still get a configure response, even when refused.
pub(super) fn keep_tiled(surface: &ToplevelSurface) {
    // The first bufferless commit supplies the prospective tile size and edges.
    // Smithay's default maximize/fullscreen callbacks send a 0x0 configure here
    // before that commit, marking initialization complete and bypassing tiling.
    if surface.is_initial_configure_sent() {
        // Explicit requests need a response even if the tile state is unchanged.
        surface.send_configure();
    }
}
