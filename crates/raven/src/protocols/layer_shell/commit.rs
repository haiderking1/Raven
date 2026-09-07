use crate::state::State;
use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{BufferAssignment, SurfaceAttributes, add_post_commit_hook, with_states},
};

struct CommitHookInstalled;

pub(super) fn already_buffered(surface: &WlSurface) -> bool {
    with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false)
        || with_states(surface, |states| {
            let mut attributes = states.cached_state.get::<SurfaceAttributes>();
            matches!(
                attributes.current().buffer,
                Some(BufferAssignment::NewBuffer(_))
            ) || matches!(
                attributes.pending().buffer,
                Some(BufferAssignment::NewBuffer(_))
            )
        })
}

pub(super) fn install(surface: &WlSurface) {
    let installed = with_states(surface, |states| {
        states
            .data_map
            .insert_if_missing_threadsafe(|| CommitHookInstalled)
    });
    if installed {
        // Keep one hook per wl_surface, including when its layer role object is recreated.
        add_post_commit_hook::<State, _>(surface, |state, _, surface| {
            let null_buffer = with_states(surface, |states| {
                matches!(
                    states
                        .cached_state
                        .get::<SurfaceAttributes>()
                        .current()
                        .buffer,
                    Some(BufferAssignment::Removed)
                )
            });
            state.layers.note_commit(surface, null_buffer);
        });
    }
}
