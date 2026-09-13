use std::collections::HashMap;

use smithay::{
    backend::input::KeyState,
    input::keyboard::{FilterResult, Keycode, Keysym, ModifiersState, keysyms},
};

use super::actions::Action;

/// The first press decides whether the entire press/release pair is swallowed.
#[derive(Debug, Default)]
pub(in crate::input) struct Shortcuts {
    pressed: HashMap<Keycode, bool>,
    pub(super) dragging: bool,
    pub(crate) bindings: super::Bindings,
}

impl Shortcuts {
    pub(super) fn filter(
        &mut self,
        keycode: Keycode,
        state: KeyState,
        modifiers: &ModifiersState,
        symbols: &[Keysym],
    ) -> FilterResult<Option<Action>> {
        let action;
        let suppressed = match state {
            KeyState::Released => {
                action = None;
                self.pressed.remove(&keycode).unwrap_or(false)
            }
            KeyState::Pressed => {
                if let Some(&suppressed) = self.pressed.get(&keycode) {
                    action = None;
                    suppressed
                } else {
                    action = if self.dragging
                        && symbols
                            .iter()
                            .any(|symbol| symbol.raw() == keysyms::KEY_Escape)
                    {
                        Some(Action::CancelWindowDrag)
                    } else {
                        self.bindings.action(modifiers, symbols)
                    };
                    self.pressed.insert(keycode, action.is_some());
                    action.is_some()
                }
            }
        };
        if suppressed {
            FilterResult::Intercept(action)
        } else {
            FilterResult::Forward
        }
    }
}

#[cfg(test)]
fn shortcut(modifiers: &ModifiersState, symbols: &[Keysym]) -> Option<Action> {
    super::Bindings::default().action(modifiers, symbols)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
