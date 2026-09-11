use crate::state::State;
use smithay::utils::IsAlive;

impl State {
    pub(crate) fn apply_fullscreen_transitions(&mut self, index: usize) {
        let workspace = &self.workspaces.entries[index];
        let full = &workspace.fullscreen;
        let mut next = full.displayed.clone().filter(|window| {
            window.alive()
                && workspace.space.element_location(window).is_some()
                && !full
                    .entries
                    .get(window)
                    .and_then(|e| e.transition.as_ref())
                    .is_some_and(|t| t.committed && !t.target.fullscreen)
        });
        if let Some(window) = full.requested.as_ref()
            && workspace.space.element_location(window).is_some()
            && full.entries.get(window).is_some_and(|e| {
                e.intent
                    && match e.transition.as_ref() {
                        Some(t) => t.committed && t.target.fullscreen,
                        None => e.applied.is_some(),
                    }
            })
        {
            next = Some(window.clone());
        }
        if self.fullscreen_area().is_none() {
            next = None;
        }
        let applying = full.displayed != next
            || full
                .entries
                .values()
                .any(|entry| entry.transition.as_ref().is_some_and(|t| t.committed));
        let hidden = if full.displayed != next {
            self.fullscreen_hidden_roots(index, next.as_ref())
        } else {
            Vec::new()
        };
        if self.fullscreen_grab_blocks(&hidden) {
            return;
        }
        for root in hidden {
            self.cancel_resize_surface(&root);
            self.dismiss_window_popups(&root);
        }
        if applying {
            self.begin_resize_batch(index);
        }
        let full = &mut self.workspaces.entries[index].fullscreen;
        let mut changed = full.displayed != next;
        full.displayed = next;
        for entry in full.entries.values_mut() {
            if entry.transition.as_ref().is_some_and(|t| t.committed) {
                let transition = entry.transition.take().unwrap();
                let applied = transition
                    .target
                    .geometry
                    .filter(|_| transition.target.fullscreen);
                changed |= entry.applied != applied;
                entry.applied = applied;
            }
        }
        if changed {
            self.retile_workspace(index);
            if index == self.workspaces.active {
                self.request_redraw();
                self.restore_focus();
                self.refresh_tiling_pointer();
            }
        }
        if applying {
            self.end_resize_batch();
        }
    }
}
