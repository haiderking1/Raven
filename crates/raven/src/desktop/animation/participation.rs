use smithay::backend::renderer::element::{
    Id, RenderElementPresentationState, RenderElementState, RenderElementStates,
};
use std::borrow::Cow;

/// The renderer computes these from actual live draws, internal opaque regions,
/// output clipping and the opaque scene above the resize group. No old snapshot
/// surface IDs are included, even when they still hold client buffer references.
pub(crate) struct GroupParticipation {
    pub group: Id,
    pub live: Vec<(Id, usize)>,
}

impl crate::state::State {
    pub(crate) fn animation_render_states<'a>(
        &self,
        states: &'a RenderElementStates,
    ) -> Cow<'a, RenderElementStates> {
        let groups = self.animations.participation.borrow();
        if groups.is_empty() {
            return Cow::Borrowed(states);
        }
        let mut expanded = states.clone();
        for group in groups.iter() {
            let shown = states
                .element_render_state(group.group.clone())
                .is_some_and(|state| {
                    state.visible_area > 0
                        && state.presentation_state != RenderElementPresentationState::Skipped
                });
            for (id, area) in &group.live {
                let visible = if shown { *area } else { 0 };
                expanded.states.insert(
                    id.clone(),
                    RenderElementState {
                        visible_area: visible,
                        presentation_state: if visible > 0 {
                            RenderElementPresentationState::Rendering { reason: None }
                        } else {
                            RenderElementPresentationState::Skipped
                        },
                    },
                );
            }
        }
        Cow::Owned(expanded)
    }
}
