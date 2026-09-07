use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{AsRenderElements, surface::WaylandSurfaceRenderElement},
        gles::GlesRenderer,
    },
    desktop::{LayerMap, layer_map_for_output},
    output::Output,
    wayland::shell::wlr_layer::Layer,
};

pub(super) fn elements(
    renderer: &mut GlesRenderer,
    state: &State,
    output: &Output,
) -> Vec<WaylandSurfaceRenderElement<GlesRenderer>> {
    let mut elements = Vec::new();
    let scale = output.current_scale().fractional_scale();
    let map = layer_map_for_output(output);
    // Front to back, matching hit testing. Smithay's space_render_elements only
    // partitions upper/lower layers, so a later Top could cover an older Overlay.
    append_layers(
        renderer,
        &map,
        &[Layer::Overlay, Layer::Top],
        scale,
        &mut elements,
    );
    if let Some(area) = state.space().output_geometry(output) {
        elements.extend(
            state
                .space()
                .render_elements_for_region(renderer, &area, scale, 1.0),
        );
    }
    append_layers(
        renderer,
        &map,
        &[Layer::Bottom, Layer::Background],
        scale,
        &mut elements,
    );
    elements
}

fn append_layers(
    renderer: &mut GlesRenderer,
    map: &LayerMap,
    levels: &[Layer],
    scale: f64,
    elements: &mut Vec<WaylandSurfaceRenderElement<GlesRenderer>>,
) {
    for level in levels {
        for layer in map.layers_on(*level).rev() {
            let Some(geometry) = map.layer_geometry(layer) else {
                continue;
            };
            // Layer geometry includes the surface-tree bounding-box offset.
            let origin = geometry.loc - layer.bbox().loc;
            elements.extend(
                layer.render_elements::<WaylandSurfaceRenderElement<GlesRenderer>>(
                    renderer,
                    origin.to_physical_precise_round(scale),
                    scale.into(),
                    1.0,
                ),
            );
        }
    }
}
