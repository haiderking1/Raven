use super::{SceneElement, clipping};
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{Kind, surface::render_elements_from_surface_tree},
        gles::GlesRenderer,
    },
    desktop::PopupManager,
    utils::{Logical, Rectangle},
};

pub(super) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    area: Rectangle<i32, Logical>,
    scale: f64,
    elements: &mut Vec<SceneElement>,
) {
    let output_clip = Rectangle::from_size(area.size);
    for window in state.visible_windows().rev() {
        let Some(toplevel) = window.toplevel() else {
            continue;
        };
        let Some(origin) = state.window_surface_origin(window) else {
            continue;
        };
        let location = (origin - area.loc).to_physical_precise_round(scale);
        // Smithay 0.7 Window::render_elements takes a buffer origin. Its popup
        // offset includes the root geometry and subtracts the popup geometry.
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
        let Some(mut clip) = state
            .window_layout_geometry(window)
            .and_then(|allocation| allocation.intersection(area))
        else {
            continue;
        };
        clip.loc -= area.loc;
        let surfaces = render_elements_from_surface_tree(
            renderer,
            toplevel.wl_surface(),
            location,
            scale,
            1.0,
            Kind::Unspecified,
        );
        clipping::append(elements, surfaces, clip, scale);
    }
}
