use super::*;

#[test]
fn shortcuts_swallow_repeat_and_release_after_modifiers_are_released() {
    for (keycode, symbol, shift, expected) in [
        (24, keysyms::KEY_q, false, Action::LaunchTerminal),
        (24, keysyms::KEY_q, true, Action::Quit),
        (54, keysyms::KEY_c, false, Action::CloseWindow),
        (40, keysyms::KEY_d, false, Action::LaunchFuzzel),
        (41, keysyms::KEY_f, false, Action::ToggleFullscreen),
        (41, keysyms::KEY_F, false, Action::ToggleFullscreen),
        (10, keysyms::KEY_1, false, Action::SwitchWorkspace(0)),
        (18, keysyms::KEY_9, false, Action::SwitchWorkspace(8)),
        (19, keysyms::KEY_0, false, Action::SwitchWorkspace(9)),
        (10, keysyms::KEY_1, true, Action::MoveToWorkspace(0)),
        (18, keysyms::KEY_9, true, Action::MoveToWorkspace(8)),
        (19, keysyms::KEY_0, true, Action::MoveToWorkspace(9)),
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
        assert!(matches!(
            shortcuts.filter(key, KeyState::Pressed, &modifiers, &symbols),
            FilterResult::Intercept(Some(actual)) if actual == expected
        ));
    }
}

#[test]
fn shortcuts_reject_extra_modifiers_and_non_digit_workspace_symbols() {
    for shift in [false, true] {
        for (logo, ctrl, alt) in [
            (false, false, false),
            (false, true, true),
            (true, true, false),
            (true, false, true),
            (true, true, true),
        ] {
            let modifiers = ModifiersState {
                logo,
                ctrl,
                alt,
                shift,
                ..Default::default()
            };
            for symbol in [
                keysyms::KEY_1,
                keysyms::KEY_9,
                keysyms::KEY_0,
                keysyms::KEY_d,
                keysyms::KEY_f,
                keysyms::KEY_F,
            ] {
                assert_eq!(shortcut(&modifiers, &[Keysym::new(symbol)]), None);
            }
        }
        let modifiers = ModifiersState {
            logo: true,
            shift,
            ..Default::default()
        };
        if shift {
            for symbol in [keysyms::KEY_d, keysyms::KEY_f, keysyms::KEY_F] {
                assert_eq!(shortcut(&modifiers, &[Keysym::new(symbol)]), None);
            }
        }
        for symbol in [
            keysyms::KEY_slash,
            keysyms::KEY_colon,
            keysyms::KEY_exclam,
            keysyms::KEY_parenright,
        ] {
            assert_eq!(shortcut(&modifiers, &[Keysym::new(symbol)]), None);
        }
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
            shortcuts.filter(key, KeyState::Pressed, &modifiers, &symbols),
            FilterResult::Intercept(None)
        ));
        assert!(matches!(
            shortcuts.filter(key, KeyState::Released, &modifiers, &symbols),
            FilterResult::Intercept(None)
        ));
    }
}
