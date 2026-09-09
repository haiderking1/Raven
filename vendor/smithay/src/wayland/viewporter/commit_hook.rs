//! Per-surface hook lifetime, independent of lazily allocated viewport state.

use wayland_server::protocol::wl_surface::WlSurface;

use super::{compositor, viewport_pre_commit_hook, with_states};

struct ViewportCommitHook;

pub(super) fn ensure_registered<D: 'static>(surface: &WlSurface) {
    let initial = with_states(surface, |states| {
        states
            .data_map
            .insert_if_missing_threadsafe(|| ViewportCommitHook)
    });
    if initial {
        compositor::add_pre_commit_hook::<D, _>(surface, viewport_pre_commit_hook);
    }
}
