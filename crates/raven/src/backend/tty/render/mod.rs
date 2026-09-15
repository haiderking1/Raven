mod animation;
mod borders;
mod clipping;
pub(super) mod cursor;
mod desktop;
mod dragging;
mod layers;
mod outcome;
mod pointer;
pub(super) mod recovery;
mod rounded;
mod windows;
pub(super) use outcome::RenderOutcome;
mod screenshot;
mod submit;
mod switcher;

use super::{device::Device, dmabuf::FeedbackDelivery};
use crate::state::State;
use smithay::backend::renderer::{
    element::{
        memory::MemoryRenderBufferRenderElement, render_elements,
        surface::WaylandSurfaceRenderElement, utils::CropRenderElement,
    },
    gles::GlesRenderer,
};
use std::error::Error;

render_elements! {
    SceneElement<=GlesRenderer>;
    Surface=WaylandSurfaceRenderElement<GlesRenderer>,
    ClippedSurface=CropRenderElement<WaylandSurfaceRenderElement<GlesRenderer>>,
    Memory=MemoryRenderBufferRenderElement<GlesRenderer>,
    Border=borders::BorderElement,
    Rounded=rounded::Rounded,
    RoundedBorder=rounded::Ring,
    Screenshot=screenshot::View,
    ResizeLive=animation::LiveElement,
    ResizeBlend=animation::Blend,
}

pub(super) struct Scene {
    pub(in crate::backend::tty) cursor: cursor::Cursors,
    feedback: FeedbackDelivery,
    switcher: switcher::Overlay,
    screenshot: screenshot::Capture,
    elements: Vec<SceneElement>,
    pub(in crate::backend::tty) animations: animation::Animations,
}

impl Scene {
    pub fn new(prepared: Option<cursor::PreparedCursor>) -> std::io::Result<Self> {
        Ok(Self {
            cursor: match prepared {
                Some(prepared) => cursor::Cursors::prepared(prepared),
                None => cursor::Cursors::new()?,
            },
            feedback: FeedbackDelivery::default(),
            switcher: Default::default(),
            screenshot: Default::default(),
            elements: Vec::new(),
            animations: animation::Animations::default(),
        })
    }

    pub(in crate::backend::tty) fn clear_screenshot(&mut self) {
        self.screenshot = Default::default();
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
        pointer::append(
            &mut device.renderer,
            state,
            output,
            &mut self.cursor,
            elements,
        )?;
        self.screenshot
            .append(&mut device.renderer, state, output, elements)?;
        self.switcher
            .append(&mut device.renderer, state, output, elements)?;
        // Desktop elements include layer shells, XDG popups, and subsurface trees.
        desktop::append(
            &mut device.renderer,
            state,
            output,
            &mut self.animations,
            elements,
        )?;
        animation::accounting::record(state, output, elements);
        let outcome = submit::render(device, state, elements, &mut self.feedback, deferred)?;
        self.screenshot
            .after_frame(&mut device.renderer, state, elements);
        Ok(outcome)
    }
}

impl super::TtyBackend {
    pub(crate) fn poll_screenshot(
        &mut self,
    ) -> Option<(
        u64,
        Result<crate::desktop::screenshot::encoding::Pixels, String>,
    )> {
        if !self.input_active() {
            return None;
        }
        self.scene
            .screenshot
            .poll(&mut self.device.as_mut()?.renderer)
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
        )
        .unwrap();
    }
    elements.len()
}
