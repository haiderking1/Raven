use crate::state::State;
use smithay::{desktop::Window, utils::IsAlive};
impl State {
    pub(super) fn request_managed_fullscreen(&mut self, window: &Window) {
        self.activate_managed_window(window);
        if self.focused_window().as_ref() == Some(window)
            || self.switcher.pending.as_ref() == Some(window)
        {
            self.foreign_toplevel.pending_fullscreen = Some(window.clone());
            self.refresh_managed_fullscreen();
        }
    }
    pub(super) fn refresh_managed_fullscreen(&mut self) {
        let Some(window) = self.foreign_toplevel.pending_fullscreen.clone() else {
            return;
        };
        if !window.alive()
            || self.workspaces.index_of(&window) != Some(self.workspaces.active)
            || self.screenshot.active()
            || self.exclusive_keyboard_layer().is_some()
            || self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
            || self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
        {
            self.foreign_toplevel.pending_fullscreen = None;
            return;
        }
        if self.focused_window().as_ref() == Some(&window) && self.window_is_visible(&window) {
            self.foreign_toplevel.pending_fullscreen = None;
            if let Some(top) = window.toplevel() {
                self.request_fullscreen(top, true);
            }
        } else if self.switcher.pending.as_ref() != Some(&window) {
            self.foreign_toplevel.pending_fullscreen = None;
        }
    }
}
