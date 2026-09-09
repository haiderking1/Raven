use crate::state::State;
use smithay::{desktop::Window, utils::IsAlive};

impl State {
    pub(crate) fn refresh_fullscreen(&mut self) {
        let has_output = self.fullscreen_area().is_some();
        for index in 0..self.workspaces.entries.len() {
            let workspace = &mut self.workspaces.entries[index];
            let full = &mut workspace.fullscreen;
            full.entries.retain(|window, _| window.alive());
            if full
                .requested
                .as_ref()
                .is_some_and(|w| !w.alive() || workspace.space.element_location(w).is_none())
            {
                full.requested = None;
            }
            let stale_display = full
                .displayed
                .as_ref()
                .is_some_and(|w| !w.alive() || workspace.space.element_location(w).is_none());
            if stale_display {
                full.displayed = None;
            }
            if !has_output {
                full.requested = None;
                for entry in full.entries.values_mut() {
                    entry.intent = false;
                }
            }
            let windows: Vec<Window> = full.entries.keys().cloned().collect();
            for window in windows {
                self.configure_fullscreen(&window, false);
            }
            self.apply_fullscreen_transitions(index);
            self.position_fullscreen_windows(index);
            if stale_display && index == self.workspaces.active {
                self.request_redraw();
                self.restore_focus();
                self.refresh_tiling_pointer();
            }
        }
    }
}
