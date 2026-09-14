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
    pub(super) switching: bool,
    pub(super) screenshot: bool,
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
                    action = if self.screenshot {
                        let bound = self.bindings.action(modifiers, symbols);
                        if matches!(bound, Some(Action::Quit | Action::SwitchVt(_))) {
                            bound
                        } else if symbols.iter().any(|s| s.raw() == keysyms::KEY_Escape) {
                            Some(Action::CancelScreenshot)
                        } else if symbols.iter().any(|s| {
                            matches!(
                                s.raw(),
                                keysyms::KEY_Return | keysyms::KEY_KP_Enter | keysyms::KEY_space
                            )
                        }) {
                            Some(Action::ConfirmScreenshot)
                        } else {
                            None
                        }
                    } else if self.switching {
                        let mut capture_modifiers = *modifiers;
                        capture_modifiers.alt = false;
                        let capture = self
                            .bindings
                            .action(&capture_modifiers, symbols)
                            .filter(|a| *a == Action::Screenshot)
                            .or_else(|| {
                                symbols
                                    .iter()
                                    .any(|s| s.raw() == keysyms::KEY_Print)
                                    .then(|| {
                                        self.bindings.action(&ModifiersState::default(), symbols)
                                    })
                                    .flatten()
                                    .filter(|a| *a == Action::Screenshot)
                            });
                        let bound = capture.or_else(|| self.bindings.action(modifiers, symbols));
                        if matches!(
                            bound,
                            Some(
                                Action::Quit
                                    | Action::SwitchVt(_)
                                    | Action::CycleApplications(_)
                                    | Action::Screenshot
                            )
                        ) {
                            bound
                        } else if symbols.iter().any(|s| s.raw() == keysyms::KEY_Escape) {
                            Some(Action::CancelAppSwitcher)
                        } else if symbols.iter().any(|s| {
                            s.raw() == keysyms::KEY_Tab
                                || s.raw() == keysyms::KEY_Right
                                || s.raw() == keysyms::KEY_Left
                        }) {
                            Some(Action::CycleApplications(
                                modifiers.shift
                                    || symbols.iter().any(|s| s.raw() == keysyms::KEY_Left),
                            ))
                        } else {
                            None
                        }
                    } else if self.dragging
                        && symbols
                            .iter()
                            .any(|symbol| symbol.raw() == keysyms::KEY_Escape)
                    {
                        Some(Action::CancelWindowDrag)
                    } else {
                        self.bindings.action(modifiers, symbols)
                    };
                    let owned = self.screenshot || self.switching || action.is_some();
                    self.pressed.insert(keycode, owned);
                    owned
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
