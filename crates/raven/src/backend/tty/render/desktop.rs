use super::{SceneElement, layers, windows};
use crate::state::State;
use smithay::{
    backend::renderer::gles::GlesRenderer, output::Output, utils::Rectangle,
    wayland::shell::wlr_layer::Layer,
};

pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    output: &Output,
    animations: &mut super::animation::Animations,
    elements: &mut Vec<SceneElement>,
) {
    let Some(area) = state.space().output_geometry(output) else {
        return;
    };
    let scale = output.current_scale().fractional_scale();
    let clip = Rectangle::from_size(area.size);
    // Front to back, in the same order as State::surface_under. Each layer
    // pass owns its lock: window clipping reads the layer-reserved workarea
    // and must never run under an already-held LayerMap guard.
    layers::append(
        renderer,
        state,
        output,
        &[Layer::Overlay, Layer::Top],
        scale,
        clip,
        elements,
    );
    windows::append(renderer, state, area, scale, animations, elements);
    layers::append(
        renderer,
        state,
        output,
        &[Layer::Bottom, Layer::Background],
        scale,
        clip,
        elements,
    );
}

#[cfg(test)]
mod tests;
