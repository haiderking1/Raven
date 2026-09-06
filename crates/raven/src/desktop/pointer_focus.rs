use crate::state::State;
use smithay::utils::{Logical, Point};

impl State {
    pub(crate) fn focus_window_on_motion(&mut self, point: Point<f64, Logical>) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        // Selection drags, DnD, and popup grabs retain their existing focus.
        if keyboard.is_grabbed()
            || self
                .seat
                .get_pointer()
                .is_some_and(|pointer| pointer.is_grabbed())
        {
            return;
        }
        let Some((window, _)) = self.space().element_under(point) else {
            // Moving across background should not stop typing into a window.
            return;
        };
        let Some(toplevel) = window.toplevel() else {
            return;
        };
        if keyboard.current_focus().as_ref() == Some(toplevel.wl_surface()) {
            // Ordinary motion within the focused tile must not reconfigure
            // clients or resend keyboard focus at the mouse's polling rate.
            return;
        }
        self.activate_window(Some(window.clone()));
    }
}
