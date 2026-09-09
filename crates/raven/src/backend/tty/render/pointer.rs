use super::{SceneElement, clipping};
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{
            Kind,
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            surface::render_elements_from_surface_tree,
        },
        gles::GlesRenderer,
    },
    input::pointer::{CursorImageStatus, CursorImageSurfaceData},
    output::Output,
    reexports::wayland_server::Resource,
    utils::{Logical, Physical, Point, Rectangle},
    wayland::compositor::with_states,
};

pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &mut State,
    output: &Output,
    cursor: &MemoryRenderBuffer,
    elements: &mut Vec<SceneElement>,
) -> Result<(), smithay::backend::renderer::gles::GlesError> {
    let Some(area) = state.space().output_geometry(output) else {
        return Ok(());
    };
    let scale = output.current_scale().fractional_scale();
    let pointer = state.pointer_location - area.loc.to_f64();
    if matches!(&state.cursor_status, CursorImageStatus::Surface(surface) if !surface.is_alive()) {
        state.cursor_status = CursorImageStatus::default_named();
    }
    match &state.cursor_status {
        CursorImageStatus::Hidden => {}
        CursorImageStatus::Named(_) => {
            // All named shapes use the built-in arrow. Its tip is at 2,2.
            let location = (pointer - Point::<f64, Logical>::from((2.0, 2.0))).to_physical(scale);
            elements.push(
                MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    location,
                    cursor,
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
            // Leave cursor-plane clipping to DRM so partially off-output cursors
            // retain the hardware cursor path and its software fallback.
            elements.extend(render_elements_from_surface_tree(
                renderer,
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
        let surfaces = render_elements_from_surface_tree(
            renderer,
            icon,
            location,
            scale,
            1.0,
            Kind::Unspecified,
        );
        clipping::append(elements, surfaces, Rectangle::from_size(area.size), scale);
    }
    Ok(())
}
