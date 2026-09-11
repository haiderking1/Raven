mod tree;

use crate::state::State;
use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{get_children, get_parent, with_states},
};
use std::sync::Mutex;

impl State {
    pub(crate) fn repaint_committed_subsurfaces(&self, root: &WlSurface, changed: &WlSurface) {
        let window = self
            .windows
            .iter()
            .find(|window| {
                window
                    .toplevel()
                    .is_some_and(|top| top.wl_surface() == root)
            })
            .map(|window| window.geometry());
        let (surfaces, nodes) = tree::snapshot(root);
        let geometry = tree::Geometry { window, nodes };
        let moved = with_states(root, |states| {
            states
                .data_map
                .insert_if_missing(|| Mutex::new(tree::Geometry::default()));
            let mut previous = states
                .data_map
                .get::<Mutex<tree::Geometry>>()
                .unwrap()
                .lock()
                .unwrap();
            let moved = *previous != geometry;
            *previous = geometry;
            moved
        });
        if moved {
            // Repaint the parent and siblings too: a desynchronized child can
            // move before its parent's XDG geometry changes. Smithay's element
            // tracker clears old rectangles; fresh buffer damage also redraws
            // the pixels exposed within the surviving surface tree.
            for surface in surfaces {
                with_renderer_surface_state(&surface, |state| state.damage_entire_buffer());
            }
        } else if let Some(parent) = get_parent(changed) {
            // Re-submit sibling damage on an ordinary child commit without
            // forcing every video frame to repaint the whole window or output.
            for sibling in get_children(&parent) {
                if sibling != *changed && sibling != parent {
                    with_renderer_surface_state(&sibling, |state| state.replay_buffer_damage());
                }
            }
        }
    }
}
