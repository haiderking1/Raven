use super::super::{SceneElement, borders, clipping};
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{Kind, surface::render_elements_from_surface_tree},
        gles::GlesRenderer,
    },
    desktop::{PopupManager, Window},
    utils::{Logical, Point, Rectangle},
};

/// Move the live surface tree visually without changing its tile allocation,
/// configure state, input regions, or the order that will be swapped on drop.
pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    window: &Window,
    offset: Point<i32, Logical>,
    area: Rectangle<i32, Logical>,
    scale: f64,
    elements: &mut Vec<SceneElement>,
) {
    let Some(top) = window.toplevel() else {
        return;
    };
    let Some(origin) = state.window_surface_origin(window) else {
        return;
    };
    let location = (origin + offset - area.loc).to_physical_precise_round(scale);
    for (popup, popup_offset) in PopupManager::popups_for_surface(top.wl_surface()) {
        let popup_offset = (window.geometry().loc + popup_offset - popup.geometry().loc)
            .to_physical_precise_round(scale);
        let surfaces = render_elements_from_surface_tree(
            renderer,
            popup.wl_surface(),
            location + popup_offset,
            scale,
            1.0,
            Kind::Unspecified,
        );
        clipping::append(elements, surfaces, Rectangle::from_size(area.size), scale);
    }
    if let (Some(mut frame), Some(mut client)) = (
        state.window_frame_geometry(window),
        state.window_client_geometry(window),
    ) {
        frame.loc += offset;
        client.loc += offset;
        borders::append_at(
            state, window, area, scale, true, frame, client, 1.0, elements,
        );
    }
    let Some(mut clip) = state.window_render_geometry(window) else {
        return;
    };
    clip.loc += offset;
    let Some(mut clip) = clip.intersection(area) else {
        return;
    };
    clip.loc -= area.loc;
    let surfaces = render_elements_from_surface_tree(
        renderer,
        top.wl_surface(),
        location,
        scale,
        1.0,
        Kind::Unspecified,
    );
    clipping::append(elements, surfaces, clip, scale);
}
