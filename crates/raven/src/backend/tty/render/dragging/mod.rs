mod preview;

use super::SceneElement;
use crate::state::State;
use smithay::{
    backend::renderer::gles::GlesRenderer,
    utils::{Logical, Rectangle},
};

pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    area: Rectangle<i32, Logical>,
    scale: f64,
    elements: &mut Vec<SceneElement>,
) -> Result<(), smithay::backend::renderer::gles::GlesError> {
    if let Some((window, offset)) = state.dragged_tile() {
        preview::append(renderer, state, window, offset, area, scale, elements)?;
    }
    Ok(())
}
