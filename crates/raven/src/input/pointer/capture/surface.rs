use crate::state::State;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{BufferAssignment, SurfaceAttributes, get_parent, with_states},
};

fn belongs_to(surface: &WlSurface, ancestor: &WlSurface) -> bool {
    let mut current = Some(surface.clone());
    while let Some(surface) = current {
        if &surface == ancestor {
            return true;
        }
        current = get_parent(&surface);
    }
    false
}

impl State {
    pub(crate) fn pointer_capture_surface_gone(&mut self, surface: &WlSurface) {
        // Smithay orphans children before invoking destruction hooks. Remembered
        // ancestry is necessary here; walking get_parent alone misses that case.
        let captured = self
            .input
            .capture
            .active
            .as_ref()
            .is_some_and(|a| a.ancestors.contains(surface));
        if captured {
            self.release_pointer_capture();
        }
        if captured
            || self
                .input
                .capture
                .focus
                .as_ref()
                .is_some_and(|(s, _)| belongs_to(s, surface))
        {
            self.input.capture.focus = None;
        }
    }

    pub(crate) fn pointer_capture_surface_commit(&mut self, surface: &WlSurface) {
        let removed = with_states(surface, |states| {
            matches!(
                states
                    .cached_state
                    .get::<SurfaceAttributes>()
                    .current()
                    .buffer,
                Some(BufferAssignment::Removed)
            )
        });
        if removed {
            self.pointer_capture_surface_gone(surface);
        } else {
            self.reconcile_pointer_capture();
        }
    }
}
