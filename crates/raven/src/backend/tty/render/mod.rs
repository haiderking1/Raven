mod cursor;
mod desktop;
mod outcome;
mod recovery;
pub(super) use outcome::RenderOutcome;
mod submit;

use super::{device::Device, dmabuf::FeedbackDelivery};
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{
            Kind,
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            render_elements,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
        },
        gles::GlesRenderer,
    },
    input::pointer::{CursorImageStatus, CursorImageSurfaceData},
    reexports::wayland_server::Resource,
    utils::{Logical, Physical, Point},
    wayland::compositor::with_states,
};
use std::error::Error;

render_elements! {
    SceneElement<=GlesRenderer>;
    Surface=WaylandSurfaceRenderElement<GlesRenderer>,
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

    /// Report accepted submission and plane usage; keep one KMS frame in flight.
    pub fn render(
        &mut self,
        device: &mut Device,
        state: &mut State,
    ) -> Result<RenderOutcome, Box<dyn Error>> {
        let mut elements = std::mem::take(&mut self.elements);
        let result = self.render_into(device, state, &mut elements);
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
    ) -> Result<RenderOutcome, Box<dyn Error>> {
        let output = &device.output;
        let scale = output.current_scale().fractional_scale();
        let pointer = state.pointer_location;
        if matches!(&state.cursor_status, CursorImageStatus::Surface(surface) if !surface.is_alive())
        {
            state.cursor_status = CursorImageStatus::default_named();
        }
        match &state.cursor_status {
            CursorImageStatus::Hidden => {}
            CursorImageStatus::Named(_) => {
                // All named shapes currently use the built-in arrow. Its tip is at 2,2.
                let location =
                    (pointer - Point::<f64, Logical>::from((2.0, 2.0))).to_physical(scale);
                elements.push(
                    MemoryRenderBufferRenderElement::from_buffer(
                        &mut device.renderer,
                        location,
                        &self.cursor,
                        None,
                        None,
                        None,
                        Kind::Cursor,
                    )?
                    .into(),
                );
            }
            CursorImageStatus::Surface(surface) => {
                let hotspot = with_states(surface, |states| {
                    states
                        .data_map
                        .get::<CursorImageSurfaceData>()
                        .map(|attributes| {
                            attributes
                                .lock()
                                .expect("cursor attributes poisoned")
                                .hotspot
                        })
                        .unwrap_or_default()
                });
                let location: Point<i32, Physical> =
                    (pointer - hotspot.to_f64()).to_physical_precise_round(scale);
                elements.extend(render_elements_from_surface_tree(
                    &mut device.renderer,
                    surface,
                    location,
                    scale,
                    1.0,
                    Kind::Cursor,
                ));
            }
        }
        if let Some(icon) = state.dnd_icon.as_ref().filter(|surface| surface.is_alive()) {
            let location: Point<i32, Physical> = pointer.to_physical_precise_round(scale);
            elements.extend(render_elements_from_surface_tree(
                &mut device.renderer,
                icon,
                location,
                scale,
                1.0,
                Kind::Unspecified,
            ));
        }
        // Desktop elements include layer shells, XDG popups, and subsurface trees.
        desktop::append(&mut device.renderer, state, output, elements);
        submit::render(device, state, elements, &mut self.feedback)
    }
}
