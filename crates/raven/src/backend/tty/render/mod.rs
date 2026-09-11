mod animation;
mod borders;
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
    Border=borders::BorderElement,
    ResizeLive=animation::LiveElement,
    ResizeBlend=animation::Blend,
}

pub(super) struct Scene {
    cursor: MemoryRenderBuffer,
    feedback: FeedbackDelivery,
    elements: Vec<SceneElement>,
    pub(in crate::backend::tty) animations: animation::Animations,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            cursor: cursor::default_arrow(),
            feedback: FeedbackDelivery::default(),
            elements: Vec::new(),
            animations: animation::Animations::default(),
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
        self.animations.begin_frame();
        let result = self.render_into(device, state, &mut elements, deferred);
        if let Ok(outcome) = &result {
            if outcome.queued {
                self.animations.queued();
            } else {
                self.animations.not_queued();
            }
        }
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
        desktop::append(
            &mut device.renderer,
            state,
            output,
            &mut self.animations,
            elements,
        );
        animation::accounting::record(state, output, elements);
        submit::render(device, state, elements, &mut self.feedback, deferred)
    }
}

#[cfg(test)]
pub(crate) fn test_scene_size(renderer: &mut GlesRenderer, state: &State) -> usize {
    let mut elements = Vec::new();
    if let Some(output) = &state.output {
        desktop::append(
            renderer,
            state,
            output,
            &mut animation::Animations::default(),
            &mut elements,
        );
    }
    elements.len()
}
