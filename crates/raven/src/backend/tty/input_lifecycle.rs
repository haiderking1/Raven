use crate::state::State;
use smithay::{
    backend::input::{ButtonState, InputEvent, KeyState, KeyboardKeyEvent, PointerButtonEvent},
    reexports::input::{
        Event, Libinput,
        event::{KeyboardEvent, PointerEvent},
    },
};

/// libinput_suspend queues releases for held keys and buttons before device
/// removal. Drain them now, since its fd need not wake calloop after suspension.
/// Forward no new presses or motion from the old VT. Using the regular input
/// handler also clears its shortcut press/release bookkeeping and Smithay grabs.
pub(super) fn suspend(input: &mut Libinput, state: &mut State) {
    input.suspend();
    for event in input {
        match event {
            Event::Keyboard(KeyboardEvent::Key(event)) if event.state() == KeyState::Released => {
                if state
                    .seat
                    .get_keyboard()
                    .is_some_and(|keyboard| keyboard.pressed_keys().contains(&event.key_code()))
                {
                    crate::input::handle_event(InputEvent::Keyboard { event }, state);
                }
            }
            Event::Pointer(PointerEvent::Button(event))
                if event.state() == ButtonState::Released =>
            {
                crate::input::handle_event(InputEvent::PointerButton { event }, state);
            }
            _ => {}
        }
    }
}
