pub(crate) mod capture;
pub(super) mod dragging;
mod geometry;
mod popup;
pub(super) use popup::PopupClick;
mod output;
mod refresh;
pub(super) use refresh::PointerRefresh;
mod button;
mod buttons;
mod motion;
mod scroll;
pub(super) use buttons::Buttons;
mod virtual_input;
pub(super) use virtual_input::Source;

#[cfg(test)]
mod tests;

use crate::state::State;
use smithay::{
    backend::{
        input::{
            ButtonState, PointerAxisEvent, PointerButtonEvent, PointerMotionAbsoluteEvent,
            PointerMotionEvent,
        },
        libinput::LibinputInputBackend,
    },
    input::pointer::RelativeMotionEvent,
};

pub(super) fn relative(event: impl PointerMotionEvent<LibinputInputBackend>, state: &mut State) {
    let Some(bounds) = output::bounds(state) else {
        return;
    };
    motion::send(
        state,
        geometry::clamp(state.pointer_location + event.delta(), bounds),
        event.time_msec(),
        Some(RelativeMotionEvent {
            delta: event.delta(),
            delta_unaccel: event.delta_unaccel(),
            utime: event.time(),
        }),
        true,
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
    motion::send(
        state,
        geometry::clamp(location, bounds),
        event.time_msec(),
        None,
        true,
    );
}
pub(super) fn button(event: impl PointerButtonEvent<LibinputInputBackend>, state: &mut State) {
    let source = Source::Physical(event.device().sysname().to_owned());
    if state.input.pointer_buttons.change(
        source,
        event.button_code(),
        event.state() == ButtonState::Pressed,
    ) {
        button::send(
            state,
            event.button_code(),
            event.state(),
            event.time_msec(),
            true,
        );
    }
}
pub(super) fn axis(event: impl PointerAxisEvent<LibinputInputBackend>, state: &mut State) {
    if state.screenshot.active() || state.switcher.active() {
        return;
    }
    if let Some(pointer) = state.seat.get_pointer() {
        pointer.axis(state, scroll::frame(&event));
        pointer.frame(state);
    }
}
