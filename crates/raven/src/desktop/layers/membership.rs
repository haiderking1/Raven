use crate::state::State;
use smithay::{
    desktop::layer_map_for_output, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

impl State {
    pub(super) fn forget_layer(&mut self, surface: &WlSurface) {
        self.cancel_workspace_activation_surface(surface);
        let Some(index) = self
            .layers
            .surfaces
            .iter()
            .position(|layer| layer.wl_surface() == surface)
        else {
            return;
        };
        let layer = self.layers.surfaces.remove(index);
        if let Some(entry) = self.layers.entries.remove(surface) {
            if entry.mapped {
                layer_map_for_output(&entry.output).unmap_layer(&layer);
                self.layers.changed = true;
            }
        }
        self.dismiss_window_popups(surface);
    }

    pub(crate) fn remove_layer(&mut self, surface: &WlSurface) {
        if !self.layers.entries.contains_key(surface) {
            return;
        }
        self.forget_layer(surface);
        self.refresh_layers();
    }
}
