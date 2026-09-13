mod actions;
mod bindings;
mod configuration;
pub use actions::Action;
pub use bindings::Bindings;
mod shortcuts;
pub(super) use shortcuts::Shortcuts;

use smithay::{
    backend::{input::KeyboardKeyEvent, libinput::LibinputInputBackend},
    utils::SERIAL_COUNTER,
};

use crate::state::State;

pub(super) fn handle(event: impl KeyboardKeyEvent<LibinputInputBackend>, state: &mut State) {
    let Some(keyboard) = state.seat.get_keyboard() else {
        return;
    };
    state.input.shortcuts.dragging = state.input.drag.is_some();
    let keycode = event.key_code();
    let key_state = event.state();
    // Always run through input, including intercepted releases, so XKB sees
    // every transition. Raw symbols avoid Shift+Q and Ctrl+Alt+Fn translations.
    let action = keyboard.input(
        state,
        keycode,
        key_state,
        SERIAL_COUNTER.next_serial(),
        event.time_msec(),
        |state, modifiers, keys| {
            state
                .input
                .shortcuts
                .filter(keycode, key_state, modifiers, &keys.raw_syms())
        },
    );
    if let Some(action) = action.flatten() {
        action.execute(state);
    }
}
