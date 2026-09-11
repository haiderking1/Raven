use super::Feedback;
use crate::state::State;
use smithay::{
    backend::renderer::element::{Id, RenderElementStates, utils::select_dmabuf_feedback},
    desktop::{layer_map_for_output, utils::with_surfaces_surface_tree},
    input::pointer::CursorImageStatus,
    output::Output,
    reexports::wayland_server::{
        Resource, Weak, backend::ObjectId, protocol::wl_surface::WlSurface,
    },
    wayland::{
        compositor::{SurfaceData, with_states},
        dmabuf::SurfaceDmabufFeedbackState,
    },
};
use std::collections::HashMap;

/// Remember only advised resources, without extending their lifetimes. A surface
/// that leaves the current workspace or cursor tree still needs withdrawal.
#[derive(Default)]
pub(in crate::backend::tty) struct FeedbackDelivery {
    advised: HashMap<ObjectId, Weak<WlSurface>>,
    scratch: HashMap<ObjectId, Weak<WlSurface>>,
}

impl FeedbackDelivery {
    pub fn update(
        &mut self,
        state: &State,
        output: &Output,
        rendered: &RenderElementStates,
        copied_cursor: Option<&Id>,
        feedback: &Feedback,
        primary_enabled: bool,
    ) {
        let projected = state.animation_render_states(rendered);
        let rendered = projected.as_ref();
        std::mem::swap(&mut self.advised, &mut self.scratch);
        let (advised, previous) = (&mut self.advised, &mut self.scratch);
        advised.clear();
        let mut visit = |surface: &WlSurface, data: &SurfaceData| {
            previous.remove(&surface.id());
            let Some(surface_feedback) = SurfaceDmabufFeedbackState::from_states(data) else {
                return;
            };
            let copied = copied_cursor == Some(&Id::from(surface));
            let chosen = if primary_enabled && !copied {
                select_dmabuf_feedback(surface, rendered, &feedback.render, &feedback.scanout)
            } else {
                &feedback.render
            };
            surface_feedback.set_feedback(chosen);
            if chosen != &feedback.render {
                advised.insert(surface.id(), surface.downgrade());
            }
        };
        for window in state.space().elements() {
            window.with_surfaces(&mut visit);
        }
        for layer in layer_map_for_output(output).layers() {
            layer.with_surfaces(&mut visit);
        }
        if let Some(icon) = &state.dnd_icon {
            with_surfaces_surface_tree(icon, &mut visit);
        }
        if let CursorImageStatus::Surface(cursor) = &state.cursor_status {
            with_surfaces_surface_tree(cursor, &mut visit);
        }
        // Do not use a visible-only primary-output closure here: it would skip
        // precisely the hidden/unmapped surfaces whose preferences are stale.
        for surface in previous.drain().filter_map(|(_, weak)| weak.upgrade().ok()) {
            with_states(&surface, |data| {
                if let Some(surface_feedback) = SurfaceDmabufFeedbackState::from_states(data) {
                    surface_feedback.set_feedback(&feedback.render);
                }
            });
        }
    }
}
