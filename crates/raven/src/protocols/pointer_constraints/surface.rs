use crate::state::State;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{add_destruction_hook, add_post_commit_hook, get_parent, with_states},
};

struct Installed;

pub(super) fn install(surface: &WlSurface) {
    let mut current = Some(surface.clone());
    while let Some(surface) = current {
        let installed = with_states(&surface, |states| {
            states.data_map.insert_if_missing_threadsafe(|| Installed)
        });
        if installed {
            add_destruction_hook::<State, _>(&surface, |state, surface| {
                state.pointer_capture_surface_gone(surface);
            });
            // Ancestor null-buffer commits also unmap constrained subsurfaces.
            add_post_commit_hook::<State, _>(&surface, |state, _, surface| {
                state.pointer_capture_surface_commit(surface);
            });
        }
        current = get_parent(&surface);
    }
}
