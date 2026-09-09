use crate::state::State;
use smithay::{
    input::pointer::{MotionEvent, PointerHandle},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, Serial},
};

type Focus = Option<(WlSurface, Point<f64, Logical>)>;

#[derive(Debug)]
struct Snapshot {
    focus: Focus,
    location: Point<f64, Logical>,
    enter: Option<Serial>,
}

/// Smithay exposes the focused surface, but not its last supplied global origin.
#[derive(Debug, Default)]
pub(in crate::input) struct PointerRefresh {
    last: Option<Snapshot>,
}

impl PointerRefresh {
    fn unchanged(
        &self,
        pointer: &PointerHandle<State>,
        focus: &Focus,
        location: Point<f64, Logical>,
    ) -> bool {
        let Some(last) = &self.last else {
            return false;
        };
        last.focus == *focus
            && last.location == location
            && !pointer.is_grabbed()
            && pointer.current_location() == location
            && pointer.current_focus().as_ref() == focus.as_ref().map(|(surface, _)| surface)
            && pointer.last_enter() == last.enter
    }

    fn record(
        &mut self,
        pointer: &PointerHandle<State>,
        focus: Focus,
        location: Point<f64, Logical>,
    ) {
        // Grabs can redirect focus and need updates to their pending hit target.
        // Their events must stay inside Smithay's grab dispatch, not this filter.
        self.last = if !pointer.is_grabbed()
            && pointer.current_focus().as_ref() == focus.as_ref().map(|(surface, _)| surface)
            && pointer.current_location() == location
        {
            Some(Snapshot {
                focus,
                location,
                enter: pointer.last_enter(),
            })
        } else {
            None
        };
    }
}

impl State {
    /// Re-hit-test after scene changes without pretending that every redraw moved the mouse.
    /// Return whether the caller needs to include a motion/enter/leave in its pointer frame.
    pub(crate) fn refresh_pointer_focus(&mut self, serial: Serial, time: u32) -> bool {
        let Some(pointer) = self.seat.get_pointer() else {
            return false;
        };
        self.reconcile_pointer_capture();
        if self.pointer_is_locked() {
            return false;
        }
        let focus = self.surface_under(self.pointer_location);
        if self
            .input
            .pointer_refresh
            .unchanged(&pointer, &focus, self.pointer_location)
        {
            return false;
        }
        self.send_pointer_motion(
            focus,
            &MotionEvent {
                location: self.pointer_location,
                serial,
                time,
            },
        );
        true
    }

    /// Hardware motion is always dispatched. Recording it also prevents the next
    /// client redraw from echoing the same motion back to that client.
    pub(crate) fn send_pointer_motion(&mut self, focus: Focus, event: &MotionEvent) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.motion(self, focus.clone(), event);
        self.input
            .pointer_refresh
            .record(&pointer, focus, event.location);
    }
}
