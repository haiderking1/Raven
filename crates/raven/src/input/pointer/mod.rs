pub(crate) mod capture;
mod geometry;
mod output;
mod refresh;
pub(super) use refresh::PointerRefresh;
mod scroll;

#[cfg(test)]
mod tests;

use smithay::{
    backend::{
        input::{
            ButtonState, PointerAxisEvent, PointerButtonEvent, PointerMotionAbsoluteEvent,
            PointerMotionEvent,
        },
        libinput::LibinputInputBackend,
    },
    input::pointer::{ButtonEvent, MotionEvent, RelativeMotionEvent},
    utils::{Logical, Point, SERIAL_COUNTER},
};

use crate::state::State;

pub(super) fn relative(event: impl PointerMotionEvent<LibinputInputBackend>, state: &mut State) {
    let Some(bounds) = output::bounds(state) else {
        return;
    };
    let location = geometry::clamp(state.pointer_location + event.delta(), bounds);
    motion(
        state,
        location,
        event.time_msec(),
        Some(RelativeMotionEvent {
            delta: event.delta(),
            delta_unaccel: event.delta_unaccel(),
            utime: event.time(),
        }),
    );
}

pub(super) fn absolute(
    event: impl PointerMotionAbsoluteEvent<LibinputInputBackend>,
    state: &mut State,
) {
    let Some(bounds) = output::bounds(state) else {
        return;
    };
    let location = bounds.loc.to_f64() + event.position_transformed(bounds.size);
    motion(
        state,
        geometry::clamp(location, bounds),
        event.time_msec(),
        None,
    );
}

fn motion(
    state: &mut State,
    location: Point<f64, Logical>,
    time: u32,
    relative: Option<RelativeMotionEvent>,
) {
    state.reconcile_pointer_capture();
    let location = state.captured_pointer_location(location);
    let moved = state.pointer_location != location;
    let Some(pointer) = state.seat.get_pointer() else {
        return;
    };
    if state.pointer_is_locked() {
        if let Some(event) = relative {
            pointer.relative_motion(state, state.input.capture.focus.clone(), &event);
        }
        pointer.frame(state);
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
    pointer.frame(state);
}

pub(super) fn button(event: impl PointerButtonEvent<LibinputInputBackend>, state: &mut State) {
    let Some(pointer) = state.seat.get_pointer() else {
        return;
    };
    state.reconcile_pointer_capture();
    let serial = SERIAL_COUNTER.next_serial();
    let location = state.pointer_location;
    let keyboard_grabbed = state
        .seat
        .get_keyboard()
        .is_some_and(|keyboard| keyboard.is_grabbed());
    // Check before button dispatch installs an implicit click grab. Never raise
    // or focus another window during popup, drag-and-drop, or ongoing clicks.
    if !pointer.is_grabbed() {
        if event.state() == ButtonState::Pressed
            && !keyboard_grabbed
            && !state.pointer_is_captured()
        {
            state.focus_window_at(location);
        }
        // A window may have appeared beneath a stationary pointer. Refresh
        // pointer focus before delivering the click, using the same frame.
        state.refresh_pointer_focus(serial, event.time_msec());
    }
    // Grab dismissal and drag completion can change the scene without a commit.
    state.request_redraw();
    pointer.button(
        state,
        &ButtonEvent {
            serial,
            time: event.time_msec(),
            button: event.button_code(),
            state: event.state(),
        },
    );
    pointer.frame(state);
}

pub(super) fn axis(event: impl PointerAxisEvent<LibinputInputBackend>, state: &mut State) {
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.axis(state, scroll::frame(&event));
        pointer.frame(state);
    }
}
