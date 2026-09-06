use crate::state::State;
use smithay::{
    desktop::{Window, WindowSurfaceType},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, SERIAL_COUNTER},
};

impl State {
    pub fn surface_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        let (window, origin) = self.space.element_under(point)?;
        window
            .surface_under(point - origin.to_f64(), WindowSurfaceType::ALL)
            .map(|(surface, offset)| (surface, (origin + offset).to_f64()))
    }

    pub fn focus_window_at(&mut self, point: Point<f64, Logical>) {
        // Popup and drag grabs control their own focus until released.
        if self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
            || self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
        {
            return;
        }
        let window = self
            .space
            .element_under(point)
            .map(|(window, _)| window.clone());
        self.activate_window(window);
    }

    pub(crate) fn activate_window(&mut self, window: Option<Window>) {
        if let Some(window) = &window {
            self.space.raise_element(window, true);
        }
        for candidate in self.space.elements() {
            candidate.set_activated(window.as_ref() == Some(candidate));
            if let Some(toplevel) = candidate.toplevel() {
                toplevel.send_pending_configure();
            }
        }
        let focus = window.and_then(|w| w.toplevel().map(|t| t.wl_surface().clone()));
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.set_focus(self, focus, SERIAL_COUNTER.next_serial());
        }
    }

    pub(crate) fn restore_focus(&mut self) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        if keyboard.is_grabbed() {
            return;
        }
        let Some(focus) = keyboard.current_focus() else {
            return;
        };
        let still_mapped = self
            .space
            .elements()
            .any(|w| w.toplevel().is_some_and(|t| t.wl_surface() == &focus));
        if !still_mapped {
            let next = self.space.elements().next_back().cloned();
            self.activate_window(next);
        }
    }
}
