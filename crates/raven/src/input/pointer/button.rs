use super::dragging;
use crate::state::State;
use smithay::{backend::input::ButtonState, input::pointer::ButtonEvent, utils::SERIAL_COUNTER};

pub(super) fn send(state: &mut State, button: u32, pressed: ButtonState, time: u32, finish: bool) {
    let Some(pointer) = state.seat.get_pointer() else {
        return;
    };
    state.reconcile_pointer_capture();
    let serial = SERIAL_COUNTER.next_serial();
    let location = state.pointer_location;
    if dragging::button(state, button, pressed, serial, time) {
        state.clear_popup_click();
        if finish {
            pointer.frame(state);
        }
        return;
    }
    let keyboard_grabbed = state
        .seat
        .get_keyboard()
        .is_some_and(|keyboard| keyboard.is_grabbed());
    // Check before button dispatch installs an implicit click grab. Never raise
    // or focus another window during popup, drag-and-drop, or ongoing clicks.
    if !pointer.is_grabbed() {
        if pressed == ButtonState::Pressed && !keyboard_grabbed && !state.pointer_is_captured() {
            state.focus_window_at(location);
        }
        // A window may have appeared beneath a stationary pointer. Refresh
        // pointer focus before delivering the click, using the same frame.
        state.refresh_pointer_focus(serial, time);
    }
    // Grab dismissal and drag completion can change the scene without a commit.
    state.request_redraw();
    state.record_popup_click(button, pressed, serial);
    pointer.button(
        state,
        &ButtonEvent {
            serial,
            time,
            button,
            state: pressed,
        },
    );
    if finish {
        pointer.frame(state);
    }
}
