use super::cycle::SurfaceFrames;
use crate::state::State;
use smithay::{backend::renderer::element::RenderElementStates, output::Output};

impl State {
    /// Publish only successful render results, including no-damage frames.
    pub(crate) fn update_render_visibility(&self, output: &Output, rendered: &RenderElementStates) {
        let projected = self.animation_render_states(rendered);
        let rendered = projected.as_ref();
        self.frame_callbacks.occluded.set(false);
        self.with_frame_surfaces(|surface, states| {
            let visible = rendered
                .element_render_state(surface)
                .is_some_and(|element| element.visible_area > 0);
            let frames = states.data_map.get_or_insert(SurfaceFrames::default);
            frames
                .0
                .lock()
                .expect("surface frame state poisoned")
                .visibility = Some((output.downgrade(), visible));
            if !visible {
                self.frame_callbacks.occluded.set(true);
            }
        });
    }
}
