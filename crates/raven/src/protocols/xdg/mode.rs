use smithay::wayland::shell::xdg::ToplevelSurface;

/// Maximize is unsupported, but must not undo a requested fullscreen mode.
/// An explicit mode request still gets a configure response when refused.
pub(super) fn reply_unchanged(surface: &ToplevelSurface) {
    // The first bufferless commit supplies the prospective tile size and edges.
    // Smithay's default maximize/fullscreen callbacks send a 0x0 configure here
    // before that commit, marking initialization complete and bypassing tiling.
    if surface.is_initial_configure_sent() {
        // Explicit requests need a response even if the tile state is unchanged.
        surface.send_configure();
    }
}
