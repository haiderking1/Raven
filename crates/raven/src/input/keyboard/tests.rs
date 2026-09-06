use super::*;

#[test]
fn shortcuts_swallow_repeat_and_release_after_modifiers_are_released() {
    for (keycode, symbol, shift, expected) in [
        (24, keysyms::KEY_q, false, Action::LaunchTerminal),
        (24, keysyms::KEY_q, true, Action::Quit),
        (54, keysyms::KEY_c, false, Action::CloseWindow),
    ] {
        let mut shortcuts = Shortcuts::default();
        let key = Keycode::new(keycode);
        let modifiers = ModifiersState {
            logo: true,
            shift,
            ..Default::default()
        };
        let symbols = [Keysym::new(symbol)];
        assert!(matches!(
            shortcuts.filter(key, KeyState::Pressed, &modifiers, &symbols),
            FilterResult::Intercept(Some(actual)) if actual == expected
        ));
        let released_modifiers = ModifiersState::default();
        assert!(matches!(
            shortcuts.filter(key, KeyState::Pressed, &released_modifiers, &symbols),
            FilterResult::Intercept(None)
        ));
        assert!(matches!(
            shortcuts.filter(key, KeyState::Released, &released_modifiers, &symbols),
            FilterResult::Intercept(None)
        ));
        assert!(matches!(
            shortcuts.filter(key, KeyState::Pressed, &released_modifiers, &symbols),
            FilterResult::Forward
        ));
        // Adding modifiers to an already forwarded key must not steal its release.
        assert!(matches!(
            shortcuts.filter(key, KeyState::Pressed, &modifiers, &symbols),
            FilterResult::Forward
        ));
        assert!(matches!(
            shortcuts.filter(key, KeyState::Released, &modifiers, &symbols),
            FilterResult::Forward
        ));
    }
}

#[test]
fn control_alt_function_keys_select_vt_only_on_press() {
    let mut shortcuts = Shortcuts::default();
    let modifiers = ModifiersState {
        ctrl: true,
        alt: true,
        ..Default::default()
    };
    for (keycode, symbol, vt) in [(67, keysyms::KEY_F1, 1), (96, keysyms::KEY_F12, 12)] {
        let key = Keycode::new(keycode);
        let symbols = [Keysym::new(symbol)];
        assert!(matches!(
            shortcuts.filter(key, KeyState::Pressed, &modifiers, &symbols),
            FilterResult::Intercept(Some(Action::SwitchVt(actual))) if actual == vt
        ));
        assert!(matches!(
            shortcuts.filter(key, KeyState::Released, &modifiers, &symbols),
            FilterResult::Intercept(None)
        ));
    }
}
