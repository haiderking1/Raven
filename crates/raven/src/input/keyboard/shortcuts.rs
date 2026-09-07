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
                    action = shortcut(modifiers, symbols);
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

fn shortcut(modifiers: &ModifiersState, symbols: &[Keysym]) -> Option<Action> {
    for symbol in symbols {
        let symbol = symbol.raw();
        if modifiers.logo && modifiers.shift && matches!(symbol, keysyms::KEY_q | keysyms::KEY_Q) {
            return Some(Action::Quit);
        }
        if modifiers.logo && !modifiers.ctrl && !modifiers.alt {
            // Raw symbols remain digits even when Shift is held.
            let workspace = match symbol {
                keysyms::KEY_1..=keysyms::KEY_9 => Some((symbol - keysyms::KEY_1) as usize),
                keysyms::KEY_0 => Some(9),
                _ => None,
            };
            if let Some(index) = workspace {
                return Some(if modifiers.shift {
                    Action::MoveToWorkspace(index)
                } else {
                    Action::SwitchWorkspace(index)
                });
            }
        }
        if modifiers.logo && !modifiers.shift && !modifiers.ctrl && !modifiers.alt {
            match symbol {
                keysyms::KEY_q | keysyms::KEY_Q => return Some(Action::LaunchTerminal),
                keysyms::KEY_c | keysyms::KEY_C => return Some(Action::CloseWindow),
                keysyms::KEY_d | keysyms::KEY_D => return Some(Action::LaunchFuzzel),
                _ => {}
            }
        }
        if modifiers.ctrl && modifiers.alt && (keysyms::KEY_F1..=keysyms::KEY_F12).contains(&symbol)
        {
            return Some(Action::SwitchVt((symbol - keysyms::KEY_F1 + 1) as i32));
        }
    }
    None
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
