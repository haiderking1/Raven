use super::{configure::configure_pending, lifecycle::has_buffer};
use crate::state::State;
use smithay::{desktop::layer_map_for_output, utils::IsAlive};

impl State {
    /// Reconcile registry membership and arrange only configured, buffer-backed clients.
    pub(crate) fn refresh_layers(&mut self) {
        let layout = self.output.as_ref().and_then(|output| {
            self.space()
                .output_geometry(output)
                .map(|geometry| (output.clone(), geometry))
        });
        if !self.layers.dirty
            && !self.layers.changed
            && self.layers.layout == layout
            && self.layers.surfaces.iter().all(IsAlive::alive)
        {
            return;
        }
        self.layers.dirty = false;
        self.layers.layout = layout;
        for layer in self.layers.surfaces.clone() {
            let surface = layer.wl_surface();
            let Some(entry) = self.layers.entries.get(surface) else {
                continue;
            };
            if !layer.alive() || self.output.as_ref() != Some(&entry.output) {
                if layer.alive() {
                    layer.layer_surface().send_close();
                }
                self.forget_layer(surface);
                continue;
            }
            if entry.mapped && !has_buffer(surface) {
                self.unmap_layer(&layer);
                continue;
            }
            let should_map = !entry.mapped
                && has_buffer(surface)
                && entry.cycle.as_ref().is_some_and(|cycle| cycle.acknowledged);
            if should_map && layer.layer_surface().ensure_configured() {
                let result = layer_map_for_output(&entry.output).map_layer(&layer);
                if result.is_err() {
                    layer.layer_surface().send_close();
                    self.forget_layer(surface);
                    continue;
                }
                if let Some(entry) = self.layers.entries.get_mut(surface) {
                    entry.mapped = true;
                }
                self.layers.changed = true;
            }
        }
        if let Some(output) = self.output.clone() {
            let zone = {
                let mut map = layer_map_for_output(&output);
                let before = map.non_exclusive_zone();
                self.layers.changed |= map.arrange();
                self.layers.changed |= before != map.non_exclusive_zone();
                map.non_exclusive_zone()
            };
            // Pending clients receive size changes, but never output.enter or exclusive zones.
            for layer in self.layers.surfaces.clone() {
                let Some(entry) = self.layers.entries.get(layer.wl_surface()) else {
                    continue;
                };
                if !entry.mapped
                    && entry.cycle.is_some()
                    && configure_pending(&layer, &output, zone).is_err()
                {
                    layer.layer_surface().send_close();
                    self.forget_layer(layer.wl_surface());
                }
            }
        }
        if std::mem::take(&mut self.layers.changed) {
            // No LayerMap guard may survive into these methods.
            self.refresh_tiling();
            self.restore_focus();
            self.refresh_tiling_pointer();
        }
    }
}
