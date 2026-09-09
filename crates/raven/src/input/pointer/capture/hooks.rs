use super::Focus;
use crate::state::State;
use smithay::input::{
    Seat,
    pointer::{MotionEvent, MotionPhase},
};

/// Smithay calls this under its pointer mutex, including grab clear/restore paths.
/// Only the supplied targets and our recorded state may be inspected here.
pub(crate) fn motion_hook(
    state: &mut State,
    _seat: &Seat<State>,
    phase: MotionPhase,
    old: &Focus,
    focus: &Focus,
    event: &MotionEvent,
) -> bool {
    match phase {
        MotionPhase::Before => {
            state.reconcile_pointer_capture();
            if state.pointer_is_captured() {
                if old.as_ref().map(|(surface, _)| surface)
                    != focus.as_ref().map(|(surface, _)| surface)
                {
                    state.release_pointer_capture();
                } else if state.pointer_is_locked()
                    || state.captured_pointer_location(event.location) != event.location
                {
                    return false;
                }
            }
        }
        MotionPhase::After => {
            state.input.capture.focus = focus.clone();
            state.pointer_location = event.location;
            state.reconcile_pointer_capture();
        }
    }
    true
}
