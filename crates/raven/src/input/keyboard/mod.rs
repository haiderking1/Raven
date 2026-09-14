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
    dispatch(state, event.key_code(), event.state(), event.time_msec());
}

pub(crate) fn dispatch(
    state: &mut State,
    keycode: smithay::input::keyboard::Keycode,
    key_state: smithay::backend::input::KeyState,
    time: u32,
) {
    let Some(keyboard) = state.seat.get_keyboard() else {
        return;
    };
    state.input.shortcuts.dragging = state.input.drag.is_some();
    state.input.shortcuts.switching = state.switcher.active();
    state.input.shortcuts.screenshot = state.screenshot.active();
    let mut confirm = false;
    let mut follows_shift = false;
    // Always run through input, including intercepted releases, so XKB sees
    // every transition. Raw symbols avoid Shift+Q and Ctrl+Alt+Fn translations.
    let action = keyboard.input(
        state,
        keycode,
        key_state,
        SERIAL_COUNTER.next_serial(),
        time,
        |state, modifiers, keys| {
            if !state.screenshot.active() {
                confirm = state.switcher_key_state(keycode, key_state, modifiers);
            }
            let symbols = keys.raw_syms();
            follows_shift = symbols
                .iter()
                .any(|s| s.raw() == smithay::input::keyboard::keysyms::KEY_Tab);
            state
                .input
                .shortcuts
                .filter(keycode, key_state, modifiers, &symbols)
        },
    );
    if confirm {
        state.confirm_app_switcher();
    } else if let Some(action) = action.flatten() {
        let cycling = matches!(action, Action::CycleApplications(_));
        action.execute(state);
        if cycling {
            state.start_switcher_repeat(keycode, follows_shift);
        }
    }
}
