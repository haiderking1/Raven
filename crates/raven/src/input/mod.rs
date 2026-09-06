//! Input from the active libinput source. The backend suspends that source while
//! the session is inactive; do not dispatch queued events from an inactive source.

mod keyboard;
mod pointer;

use smithay::backend::{input::InputEvent, libinput::LibinputInputBackend};

use crate::state::State;

/// Persistent key disposition, kept until each key's matching release.
#[derive(Debug, Default)]
pub struct InputState {
    shortcuts: keyboard::Shortcuts,
}

/// Dispatch an event from the backend's active libinput source.
pub fn handle_event(event: InputEvent<LibinputInputBackend>, state: &mut State) {
    match event {
        InputEvent::Keyboard { event } => keyboard::handle(event, state),
        InputEvent::PointerMotion { event } => pointer::relative(event, state),
        InputEvent::PointerMotionAbsolute { event } => pointer::absolute(event, state),
        InputEvent::PointerButton { event } => pointer::button(event, state),
        InputEvent::PointerAxis { event } => pointer::axis(event, state),
        _ => {}
    }
}
