use crate::state::State;
use smithay::{
    backend::renderer::element::{Id, RenderElementPresentationState, RenderElementStates},
    desktop::{
        layer_map_for_output,
        utils::{
            OutputPresentationFeedback, surface_presentation_feedback_flags_from_states,
            take_presentation_feedback_surface_tree,
        },
    },
    input::pointer::CursorImageStatus,
    output::Output,
    reexports::{
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    wayland::compositor::SurfaceData,
};

impl State {
    /// Snapshot only feedback belonging to surfaces in this rendered frame.
    /// Hidden workspaces and occluded surfaces must not be reported as shown.
    pub(crate) fn take_presentation_feedback(
        &self,
        output: &Output,
        rendered: &RenderElementStates,
        copied_cursor: Option<&Id>,
    ) -> OutputPresentationFeedback {
        let projected = self.animation_render_states(rendered);
        let rendered = projected.as_ref();
        let mut feedback = OutputPresentationFeedback::new(output);
        let primary = |surface: &WlSurface, _: &SurfaceData| {
            rendered
                .element_render_state(surface)
                .is_some_and(|state| {
                    state.visible_area > 0
                        && state.presentation_state != RenderElementPresentationState::Skipped
                })
                .then(|| output.clone())
        };
        let flags = |surface: &WlSurface, _: &SurfaceData| {
            let mut flags = surface_presentation_feedback_flags_from_states(surface, rendered);
            // Smithay 0.7 labels the copied cursor BO as ZeroCopy too. Only
            // genuine client-buffer scanout may carry that protocol flag.
            if copied_cursor == Some(&Id::from(surface)) {
                flags.remove(wp_presentation_feedback::Kind::ZeroCopy);
            }
            flags
        };
        for window in self.space().elements() {
            window.take_presentation_feedback(&mut feedback, primary, flags);
        }
        for layer in layer_map_for_output(output).layers() {
            layer.take_presentation_feedback(&mut feedback, primary, flags);
        }
        if let Some(icon) = &self.dnd_icon {
            take_presentation_feedback_surface_tree(icon, &mut feedback, primary, flags);
        }
        if let CursorImageStatus::Surface(cursor) = &self.cursor_status {
            take_presentation_feedback_surface_tree(cursor, &mut feedback, primary, flags);
        }
        feedback
    }
}
