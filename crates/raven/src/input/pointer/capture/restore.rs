use super::region::Area;
use crate::state::State;
use smithay::{
    input::pointer::MotionEvent,
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Clock, Logical, Monotonic, Point, SERIAL_COUNTER},
};

impl State {
    pub(crate) fn pointer_capture_hint(&mut self, surface: &WlSurface, hint: Point<f64, Logical>) {
        if let Some(active) = self
            .input
            .capture
            .active
            .as_mut()
            .filter(|a| a.locked && &a.surface == surface)
        {
            active.hint = Some(hint);
        }
    }

    /// Called only after protocol removal has released its mutex, never on forced focus loss.
    pub(crate) fn pointer_capture_destroyed(&mut self, surface: &WlSurface) {
        if !self
            .input
            .capture
            .active
            .as_ref()
            .is_some_and(|a| &a.surface == surface)
        {
            return;
        }
        let active = self.input.capture.active.take().unwrap();
        let Some(hint) = active.hint.filter(|_| active.locked) else {
            return;
        };
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        if self.input.capture.suspended
            || !surface.is_alive()
            || pointer.is_grabbed()
            || pointer.current_focus().as_ref() != Some(surface)
            || self.input.capture.focus.as_ref() != Some(&(surface.clone(), active.origin))
        {
            return;
        }
        let Some(area) = Area::new(surface, None) else {
            return;
        };
        if !area.contains(hint) {
            return;
        }
        let location = active.origin + hint;
        if super::super::output::bounds(self)
            .is_some_and(|bounds| !bounds.to_f64().contains(location))
            || self.surface_under(self.pointer_location).as_ref()
                != Some(&(surface.clone(), active.origin))
            || self.surface_under(location).as_ref() != Some(&(surface.clone(), active.origin))
        {
            return;
        }
        if location == self.pointer_location {
            return;
        }
        self.send_pointer_motion(
            Some((surface.clone(), active.origin)),
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time: Clock::<Monotonic>::new().now().as_millis(),
            },
        );
        pointer.frame(self);
        self.request_redraw();
    }
}
