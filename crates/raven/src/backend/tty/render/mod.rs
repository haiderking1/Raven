mod clipping;
mod cursor;
mod desktop;
mod layers;
mod outcome;
mod pointer;
pub(super) mod recovery;
mod windows;
pub(super) use outcome::RenderOutcome;
mod submit;

use super::{device::Device, dmabuf::FeedbackDelivery};
use crate::state::State;
use smithay::backend::renderer::{
    element::{
        memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
        render_elements,
        surface::WaylandSurfaceRenderElement,
        utils::CropRenderElement,
    },
    gles::GlesRenderer,
};
use std::error::Error;

render_elements! {
    SceneElement<=GlesRenderer>;
    Surface=WaylandSurfaceRenderElement<GlesRenderer>,
    ClippedSurface=CropRenderElement<WaylandSurfaceRenderElement<GlesRenderer>>,
    Cursor=MemoryRenderBufferRenderElement<GlesRenderer>,
}

pub(super) struct Scene {
    cursor: MemoryRenderBuffer,
    feedback: FeedbackDelivery,
    elements: Vec<SceneElement>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            cursor: cursor::default_arrow(),
            feedback: FeedbackDelivery::default(),
            elements: Vec::new(),
        }
    }

    /// Render one snapshot. The scheduler reserves the pending or successor slot.
    pub fn render(
        &mut self,
        device: &mut Device,
        state: &mut State,
        deferred: bool,
    ) -> Result<RenderOutcome, Box<dyn Error>> {
        let mut elements = std::mem::take(&mut self.elements);
        let result = self.render_into(device, state, &mut elements, deferred);
        // Retain allocation capacity, never client buffer/texture references.
        elements.clear();
        self.elements = elements;
        result
    }

    fn render_into(
        &mut self,
        device: &mut Device,
        state: &mut State,
        elements: &mut Vec<SceneElement>,
        deferred: bool,
    ) -> Result<RenderOutcome, Box<dyn Error>> {
        let output = &device.output;
        pointer::append(&mut device.renderer, state, output, &self.cursor, elements)?;
        // Desktop elements include layer shells, XDG popups, and subsurface trees.
        desktop::append(&mut device.renderer, state, output, elements);
        submit::render(device, state, elements, &mut self.feedback, deferred)
    }
}

#[cfg(test)]
pub(crate) fn test_scene_size(renderer: &mut GlesRenderer, state: &State) -> usize {
    let mut elements = Vec::new();
    if let Some(output) = &state.output {
        desktop::append(renderer, state, output, &mut elements);
    }
    elements.len()
}
