use super::{SceneElement, clipping};
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{AsRenderElements, surface::WaylandSurfaceRenderElement},
        gles::GlesRenderer,
    },
    desktop::layer_map_for_output,
    output::Output,
    utils::{Logical, Rectangle},
    wayland::shell::wlr_layer::Layer,
};

pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    output: &Output,
    levels: &[Layer],
    scale: f64,
    clip: Rectangle<i32, Logical>,
    elements: &mut Vec<SceneElement>,
) {
    let map = layer_map_for_output(output);
    for level in levels {
        for layer in map.layers_on(*level).rev() {
            if !state.layer_is_visible(layer) {
                continue;
            }
            let Some(geometry) = map.layer_geometry(layer) else {
                continue;
            };
            let origin = geometry.loc - layer.bbox().loc;
            let surfaces = layer.render_elements::<WaylandSurfaceRenderElement<GlesRenderer>>(
                renderer,
                origin.to_physical_precise_round(scale),
                scale.into(),
                1.0,
            );
            clipping::append(elements, surfaces, clip, scale);
        }
    }
}
