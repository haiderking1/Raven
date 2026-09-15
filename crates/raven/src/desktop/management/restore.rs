use super::model::with;
use crate::state::State;
use smithay::desktop::Window;
impl State {
    pub(crate) fn restore_managed_window(&mut self, window: &Window) -> bool {
        let Some(index) = self.workspaces.index_of(window) else {
            return false;
        };
        if self.workspaces.entries[index]
            .space
            .element_location(window)
            .is_none()
            || self.pointer_is_captured()
        {
            return false;
        }
        let mut ancestors = Vec::new();
        let mut current = Some(window);
        while let Some(w) = current {
            ancestors.push(w.clone());
            current = self.valid_floating_parent(w);
        }
        for w in ancestors {
            with(&w, |v| v.requested = false);
        }
        self.refresh_minimized_windows();
        !self.window_is_minimized(window)
    }
    pub(crate) fn activate_managed_window(&mut self, window: &Window) {
        if self.screenshot.active()
            || self.exclusive_keyboard_layer().is_some()
            || self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
            || self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
        {
            return;
        }
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        if !self.restore_managed_window(window) {
            return;
        }
        self.switch_workspace(index);
        self.cancel_app_switcher();
        self.switcher.pending = Some(window.clone());
        self.refresh_switcher_activation();
    }
}
