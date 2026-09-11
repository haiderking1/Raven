use crate::state::State;
use smithay::utils::{Logical, Point};

impl State {
    pub fn focus_window_at(&mut self, point: Point<f64, Logical>) {
        use smithay::wayland::shell::wlr_layer::Layer;
        if self.pointer_is_captured() {
            return;
        }
        if self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
            || self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
        {
            return;
        }
        if let Some(layer) = self.exclusive_keyboard_layer() {
            self.focus_layer(&layer);
            return;
        }
        if let Some(hit) = self.layer_under(point, &[Layer::Overlay, Layer::Top]) {
            self.focus_layer(&hit.layer);
            return;
        }
        let window = self.window_focus_under(point).cloned();
        if window.is_none()
            && let Some(hit) = self.layer_under(point, &[Layer::Bottom, Layer::Background])
        {
            self.focus_layer(&hit.layer);
            return;
        }
        self.activate_window(window);
    }

    pub(crate) fn focus_window_on_motion(&mut self, point: Point<f64, Logical>) {
        if self.pointer_is_captured() {
            return;
        }
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
        if self.exclusive_keyboard_layer().is_some()
            || self
                .layer_under(
                    point,
                    &[
                        smithay::wayland::shell::wlr_layer::Layer::Overlay,
                        smithay::wayland::shell::wlr_layer::Layer::Top,
                    ],
                )
                .is_some()
        {
            return;
        }
        let Some(window) = self.window_focus_under(point) else {
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
