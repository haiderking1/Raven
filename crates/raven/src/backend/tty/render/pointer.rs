use super::{SceneElement, clipping, cursor::Cursors};
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{Kind, surface::render_elements_from_surface_tree},
        gles::GlesRenderer,
    },
    input::pointer::{CursorImageStatus, CursorImageSurfaceData},
    output::Output,
    reexports::wayland_server::Resource,
    utils::{Physical, Point, Rectangle},
    wayland::compositor::with_states,
};

pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &mut State,
    output: &Output,
    cursor: &mut Cursors,
    elements: &mut Vec<SceneElement>,
) -> Result<(), smithay::backend::renderer::gles::GlesError> {
    let Some(area) = state.space().output_geometry(output) else {
        cursor.suspend();
        return Ok(());
    };
    let scale = output.current_scale().fractional_scale();
    let pointer = state.pointer_location - area.loc.to_f64();
    if state.screenshot.active() {
        elements.push(
            cursor
                .render(
                    renderer,
                    smithay::input::pointer::CursorIcon::Crosshair,
                    pointer,
                    scale,
                )?
                .into(),
        );
        return Ok(());
    }
    if state.switcher.active() {
        elements.push(
            cursor
                .render(
                    renderer,
                    smithay::input::pointer::CursorIcon::Default,
                    pointer,
                    scale,
                )?
                .into(),
        );
        return Ok(());
    }
    if matches!(&state.cursor_status, CursorImageStatus::Surface(surface) if !surface.is_alive()) {
        state.cursor_status = CursorImageStatus::default_named();
    }
    match &state.cursor_status {
        CursorImageStatus::Hidden => cursor.suspend(),
        CursorImageStatus::Named(icon) => {
            elements.push(cursor.render(renderer, *icon, pointer, scale)?.into());
        }
        CursorImageStatus::Surface(surface) => {
            cursor.suspend();
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
