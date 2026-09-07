use super::{ConfigureCycle, configure::configure_initial};
use crate::state::State;
use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    desktop::{LayerSurface, layer_map_for_output},
    reexports::{
        wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1,
        wayland_server::{Resource, protocol::wl_surface::WlSurface},
    },
    wayland::{compositor::with_states, shell::wlr_layer::LayerSurfaceCachedState},
};

pub(super) fn has_buffer(surface: &WlSurface) -> bool {
    with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false)
}

impl State {
    pub(crate) fn commit_layer(&mut self, surface: &WlSurface) {
        let Some(layer) = self
            .layers
            .surfaces
            .iter()
            .find(|layer| layer.wl_surface() == surface)
            .cloned()
        else {
            return;
        };
        let Some(entry) = self.layers.entries.get_mut(surface) else {
            return;
        };
        self.layers.dirty = true;
        let root_commit = entry.commit.take();
        let buffered = has_buffer(surface);
        if root_commit == Some(true) && entry.cycle.is_some() {
            self.unmap_layer(&layer);
            self.refresh_layers();
            return;
        }
        if buffered && !entry.cycle.as_ref().is_some_and(|cycle| cycle.acknowledged) {
            layer.layer_surface().shell_surface().post_error(
                zwlr_layer_surface_v1::Error::InvalidSurfaceState,
                "a layer buffer requires an acknowledged configure from this mapping cycle",
            );
            return;
        }
        let initial = root_commit.is_some() && !buffered && entry.cycle.is_none();
        // Buffer/input-region changes can alter pointer focus without changing geometry.
        self.layers.changed |= entry.mapped;
        self.refresh_layers();
        if initial {
            let Some(entry) = self.layers.entries.get(surface) else {
                return;
            };
            let output = entry.output.clone();
            let zone = layer_map_for_output(&output).non_exclusive_zone();
            match configure_initial(&layer, &output, zone) {
                Ok(first) => {
                    if let Some(entry) = self.layers.entries.get_mut(surface) {
                        entry.cycle = Some(ConfigureCycle {
                            first,
                            acknowledged: false,
                        });
                    }
                }
                Err(()) => {
                    layer.layer_surface().send_close();
                    self.remove_layer(surface);
                }
            }
        }
    }

    pub(super) fn unmap_layer(&mut self, layer: &LayerSurface) {
        let Some(entry) = self.layers.entries.get_mut(layer.wl_surface()) else {
            return;
        };
        if entry.mapped {
            layer_map_for_output(&entry.output).unmap_layer(layer);
            self.layers.changed = true;
        }
        entry.mapped = false;
        entry.cycle = None;
        // Restore the creation state, including the originally requested layer.
        // Smithay resets this cache on role destruction, but not on null-buffer unmap.
        let initial = LayerSurfaceCachedState {
            layer: entry.initial_layer,
            ..Default::default()
        };
        with_states(layer.wl_surface(), |states| {
            let mut cached = states.cached_state.get::<LayerSurfaceCachedState>();
            *cached.current() = initial;
            *cached.pending() = initial;
        });
        self.dismiss_window_popups(layer.wl_surface());
    }
}
