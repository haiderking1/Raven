//! Input from the active libinput source. The backend suspends that source while
//! the session is inactive; do not dispatch queued events from an inactive source.

mod keyboard;
pub use keyboard::{Action, Bindings};
mod pointer;
pub(crate) use pointer::capture::motion_hook as pointer_motion_hook;
pub mod timing;

use smithay::backend::{input::InputEvent, libinput::LibinputInputBackend};

use crate::state::State;

/// Persistent shortcut disposition and the last dispatched pointer target.
#[derive(Debug, Default)]
pub struct InputState {
    shortcuts: keyboard::Shortcuts,
    pointer_refresh: pointer::PointerRefresh,
    popup_click: Option<pointer::PopupClick>,
    capture: pointer::capture::Capture,
    drag: Option<pointer::dragging::Drag>,
    resize_cleanup: Option<smithay::desktop::Window>,
    drag_buttons: std::collections::HashSet<u32>,
}

/// Dispatch an event from the backend's active libinput source.
pub fn handle_event(event: InputEvent<LibinputInputBackend>, state: &mut State) {
    if let Some(timing) = &mut state.input_timing {
        timing.observe_event(&event);
    }
    match event {
        InputEvent::Keyboard { event } => keyboard::handle(event, state),
        InputEvent::PointerMotion { event } => pointer::relative(event, state),
        InputEvent::PointerMotionAbsolute { event } => pointer::absolute(event, state),
        InputEvent::PointerButton { event } => pointer::button(event, state),
        InputEvent::PointerAxis { event } => pointer::axis(event, state),
        _ => {}
    }
}
