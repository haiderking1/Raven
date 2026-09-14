mod actions {
    pub use raven::runtime::settings::Action;
}
#[allow(dead_code)]
#[path = "../../src/input/keyboard/bindings/mod.rs"]
mod bindings;
use bindings::Bindings;
#[path = "../../src/input/keyboard/shortcuts.rs"]
mod shortcuts;
use actions::Action;
use shortcuts::Shortcuts;
use smithay::{
    backend::input::KeyState,
    input::keyboard::{FilterResult, Keycode, Keysym, ModifiersState, keysyms},
};

#[test]
fn switcher_swallows_new_keys_but_preserves_preexisting_press_ownership() {
    let mut keys = Shortcuts::default();
    let alt = ModifiersState {
        alt: true,
        ..Default::default()
    };
    let key = Keycode::new(38);
    let a = [Keysym::new(keysyms::KEY_a)];
    assert!(matches!(
        keys.filter(key, KeyState::Pressed, &alt, &a),
        FilterResult::Forward
    ));
    let tab = Keycode::new(23);
    let symbol = [Keysym::new(keysyms::KEY_Tab)];
    assert!(matches!(
        keys.filter(tab, KeyState::Pressed, &alt, &symbol),
        FilterResult::Intercept(Some(Action::CycleApplications(false)))
    ));
    keys.switching = true;
    assert!(matches!(
        keys.filter(key, KeyState::Released, &alt, &a),
        FilterResult::Forward
    ));
    assert!(matches!(
        keys.filter(key, KeyState::Pressed, &alt, &a),
        FilterResult::Intercept(None)
    ));
    keys.switching = false;
    assert!(matches!(
        keys.filter(key, KeyState::Released, &ModifiersState::default(), &a),
        FilterResult::Intercept(None)
    ));
    assert!(matches!(
        keys.filter(tab, KeyState::Released, &ModifiersState::default(), &symbol),
        FilterResult::Intercept(None)
    ));
}
#[test]
fn reverse_selection_and_escape_are_switcher_owned() {
    let mut keys = Shortcuts::default();
    let alt_shift = ModifiersState {
        alt: true,
        shift: true,
        ..Default::default()
    };
    assert!(matches!(
        keys.filter(
            Keycode::new(23),
            KeyState::Pressed,
            &alt_shift,
            &[Keysym::new(keysyms::KEY_Tab)]
        ),
        FilterResult::Intercept(Some(Action::CycleApplications(true)))
    ));
    keys.switching = true;
    assert!(matches!(
        keys.filter(
            Keycode::new(9),
            KeyState::Pressed,
            &alt_shift,
            &[Keysym::new(keysyms::KEY_Escape)]
        ),
        FilterResult::Intercept(Some(Action::CancelAppSwitcher))
    ));
}
