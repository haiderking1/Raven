use super::{SceneElement, animation::Animations, borders, clipping};
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{Kind, surface::render_elements_from_surface_tree},
        gles::GlesRenderer,
    },
    desktop::{PopupManager, Window},
    utils::{Logical, Rectangle},
};

pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    area: Rectangle<i32, Logical>,
    scale: f64,
    animations: &mut Animations,
    elements: &mut Vec<SceneElement>,
) {
    let output_clip = Rectangle::from_size(area.size);
    let active_root = borders::active_root(state);
    for window in state.visible_windows().rev() {
        let Some(toplevel) = window.toplevel() else {
            continue;
        };
        let Some(origin) = state.window_surface_origin(window) else {
            continue;
        };
        let location = (origin - area.loc).to_physical_precise_round(scale);
        // Popups remain live at committed coordinates, outside resize snapshots.
        // Input uses this same committed popup placement and output clip.
        for (popup, offset) in PopupManager::popups_for_surface(toplevel.wl_surface()) {
            let offset = (window.geometry().loc + offset - popup.geometry().loc)
                .to_physical_precise_round(scale);
            let surfaces = render_elements_from_surface_tree(
                renderer,
                popup.wl_surface(),
                location + offset,
                scale,
                1.0,
                Kind::Unspecified,
            );
            clipping::append(elements, surfaces, output_clip, scale);
        }
        let active = active_root.as_ref() == Some(toplevel.wl_surface());
        if !animations.append(renderer, state, window, area, scale, active, elements) {
            append_body(renderer, state, window, area, scale, active, elements);
        }
    }
}

/// Shared normal and OLD-current capture path. Never includes popup trees.
pub(super) fn append_body(
    renderer: &mut GlesRenderer,
    state: &State,
    window: &Window,
    area: Rectangle<i32, Logical>,
    scale: f64,
    active: bool,
    elements: &mut Vec<SceneElement>,
) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    let Some(origin) = state.window_surface_origin(window) else {
        return;
    };
    borders::append(state, window, area, scale, active, elements);
    let Some(mut clip) = state
        .window_layout_geometry(window)
        .and_then(|allocation| allocation.intersection(area))
    else {
        return;
    };
    clip.loc -= area.loc;
    let surfaces = render_elements_from_surface_tree(
        renderer,
        toplevel.wl_surface(),
        (origin - area.loc).to_physical_precise_round(scale),
        scale,
        1.0,
        Kind::Unspecified,
    );
    clipping::append(elements, surfaces, clip, scale);
}
