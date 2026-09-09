use crate::state::State;
use smithay::{
    desktop::{LayerSurface, WindowSurfaceType, layer_map_for_output},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

pub(crate) struct LayerHit {
    pub(crate) layer: LayerSurface,
    pub(crate) surface: WlSurface,
    pub(crate) origin: Point<f64, Logical>,
}

impl State {
    // Read lifecycle membership without recursively locking Smithay's LayerMap.
    pub(crate) fn layer_is_mapped(&self, layer: &LayerSurface) -> bool {
        self.layers
            .entries
            .get(layer.wl_surface())
            .is_some_and(|entry| entry.mapped && self.output.as_ref() == Some(&entry.output))
    }

    pub(crate) fn layer_under(
        &self,
        point: Point<f64, Logical>,
        levels: &[Layer],
    ) -> Option<LayerHit> {
        let output = self.output.as_ref()?;
        let output_geometry = self.space().output_geometry(output)?;
        if !output_geometry.to_f64().contains(point) {
            return None;
        }
        let output_origin = output_geometry.loc;
        let map = layer_map_for_output(output);
        for level in levels {
            for layer in map.layers_on(*level).rev() {
                if !self.layer_is_visible(layer) {
                    continue;
                }
                let Some(geometry) = map.layer_geometry(layer) else {
                    continue;
                };
                let origin = output_origin + geometry.loc - layer.bbox().loc;
                if let Some((surface, offset)) =
                    layer.surface_under(point - origin.to_f64(), WindowSurfaceType::ALL)
                {
                    return Some(LayerHit {
                        layer: layer.clone(),
                        surface,
                        origin: (origin + offset).to_f64(),
                    });
                }
            }
        }
        None
    }
}
