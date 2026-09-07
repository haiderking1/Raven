mod cursor;
mod desktop;

use super::device::Device;
use crate::state::State;
use smithay::{
    backend::{
        drm::compositor::{FrameFlags, PrimaryPlaneElement},
        renderer::{
            element::{
                Kind,
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                render_elements,
                surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            },
            gles::GlesRenderer,
        },
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
}

impl Scene {
    pub fn new() -> Self {
        Self {
            cursor: cursor::default_arrow(),
        }
    }

    /// Returns true only when a nonempty frame was queued for a pageflip.
    pub fn render(
        &mut self,
        device: &mut Device,
        state: &mut State,
    ) -> Result<bool, Box<dyn Error>> {
        let output = &device.output;
        let scale = output.current_scale().fractional_scale();
        let pointer = state.pointer_location;
        let mut elements = Vec::<SceneElement>::new();
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
        elements.extend(
            desktop::elements(&mut device.renderer, state, output)
                .into_iter()
                .map(SceneElement::Surface),
        );
        // Compose every element with GLES. No client scanout or hardware cursor
        // planes are required, avoiding implicit cross-device buffer assumptions.
        let frame = device.compositor.render_frame(
            &mut device.renderer,
            &elements,
            [0.055, 0.065, 0.085, 1.0],
            FrameFlags::empty(),
        )?;
        if frame.needs_sync()
            && let PrimaryPlaneElement::Swapchain(element) = &frame.primary_element
        {
            element.sync.wait()?;
        }
        let changed = !frame.is_empty;
        let feedback = changed.then(|| state.take_presentation_feedback(output, &frame.states));
        drop(frame);
        if let Some(feedback) = feedback {
            device.compositor.queue_frame(feedback)?;
        }
        Ok(changed)
    }
}
