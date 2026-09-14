use crate::state::State;
use smithay::{
    input::pointer::{MotionEvent, RelativeMotionEvent},
    utils::{Logical, Point, SERIAL_COUNTER},
};

pub(super) fn send(
    state: &mut State,
    location: Point<f64, Logical>,
    time: u32,
    relative: Option<RelativeMotionEvent>,
    finish: bool,
) {
    if state.screenshot_motion(location) {
        return;
    }
    state.reconcile_pointer_capture();
    let location = state.captured_pointer_location(location);
    if state.switcher_motion(location) {
        return;
    }
    let moved = state.pointer_location != location;
    let Some(pointer) = state.seat.get_pointer() else {
        return;
    };
    if state.pointer_is_locked() {
        if let Some(event) = relative {
            pointer.relative_motion(state, state.input.capture.focus.clone(), &event);
        }
        if finish {
            pointer.frame(state);
        }
        return;
    }
    if moved {
        state.request_redraw();
        if !state.pointer_is_captured() {
            state.focus_window_on_motion(location);
        }
    }
    let focus = state.surface_under(location);
    // Keep motion inside Smithay's grab dispatch. Its leave path resets the
    // cursor through SeatHandler::cursor_image when client focus is lost.
    state.send_pointer_motion(
        focus.clone(),
        &MotionEvent {
            location,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );
    if let Some(event) = relative {
        pointer.relative_motion(state, focus, &event);
    }
    if finish {
        pointer.frame(state);
    }
}
