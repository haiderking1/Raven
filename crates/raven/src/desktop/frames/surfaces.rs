use crate::state::State;
use smithay::{
    desktop::{layer_map_for_output, utils::with_surfaces_surface_tree},
    input::pointer::CursorImageStatus,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::SurfaceData,
};

impl State {
    /// Active, mapped desktop trees only. Hidden workspaces never get fallback callbacks.
    pub(super) fn with_frame_surfaces(&self, mut visit: impl FnMut(&WlSurface, &SurfaceData)) {
        let Some(output) = &self.output else {
            return;
        };
        // Include fullscreen-suppressed trees here, not in the scene. Successful
        // render states mark them occluded for the 250ms callback path.
        for window in self.space().elements() {
            window.with_surfaces(&mut visit);
        }
        for layer in layer_map_for_output(output).layers() {
            layer.with_surfaces(&mut visit);
        }
        if let Some(icon) = &self.dnd_icon {
            with_surfaces_surface_tree(icon, &mut visit);
        }
        if let CursorImageStatus::Surface(cursor) = &self.cursor_status {
            with_surfaces_surface_tree(cursor, &mut visit);
        }
    }
}
