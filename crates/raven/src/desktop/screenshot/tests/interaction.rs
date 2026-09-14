use super::fixture::{app, fixture};
use crate::{
    desktop::{screenshot::Phase, tests::fixture::Fixture},
    input::keyboard::dispatch,
};
use smithay::{
    backend::input::{ButtonState, KeyState},
    input::keyboard::Keycode,
    utils::Rectangle,
};
fn key(f: &mut Fixture, code: u32, pressed: bool) {
    dispatch(
        &mut f.state,
        Keycode::new(code),
        if pressed {
            KeyState::Pressed
        } else {
            KeyState::Released
        },
        100,
    );
}
#[test]
fn print_and_compact_keyboard_shortcut_select_confirm_and_cancel_without_changing_focus() {
    for confirm_key in [36, 65, 104] {
        // Enter, Space, keypad Enter
        let mut f = fixture();
        let window = app(&mut f, "editor");
        key(&mut f, 107, true);
        assert!(matches!(f.state.screenshot.phase, Phase::Requested));
        key(&mut f, 107, false);
        let generation = f.state.screenshot.generation;
        f.state.screenshot_ready(generation);
        f.state.screenshot_motion((100.0, 120.0).into());
        assert!(f.state.screenshot_button(0x110, ButtonState::Pressed));
        f.state.screenshot_motion((200.0, 240.0).into());
        assert!(f.state.screenshot_button(0x110, ButtonState::Released));
        key(&mut f, confirm_key, true);
        assert!(
            matches!(f.state.screenshot.phase,Phase::Exporting(r) if r==Rectangle::new((100,120).into(),(101,121).into()))
        );
        key(&mut f, confirm_key, false);
        f.state.cancel_screenshot();
        key(&mut f, 133, true);
        key(&mut f, 50, true);
        key(&mut f, 39, true); // Super+Shift+S
        assert!(matches!(f.state.screenshot.phase, Phase::Requested));
        key(&mut f, 133, false);
        key(&mut f, 50, false);
        key(&mut f, 39, false);
        let generation = f.state.screenshot.generation;
        f.state.screenshot_ready(generation);
        assert!(f.state.screenshot_button(0x110, ButtonState::Pressed));
        key(&mut f, 9, true);
        key(&mut f, 9, false);
        assert!(!f.state.screenshot.active());
        assert!(
            f.state.screenshot_button(0x110, ButtonState::Released),
            "late release belongs to the dismissed selector"
        );
        assert!(!f.state.screenshot_button(0x110, ButtonState::Released));
        assert_eq!(f.state.focused_window(), Some(window));
    }
}
#[test]
fn capturing_alt_tab_does_not_confirm_the_app_when_alt_is_released() {
    let mut f = fixture();
    app(&mut f, "browser");
    let terminal = app(&mut f, "terminal");
    key(&mut f, 64, true);
    key(&mut f, 23, true); // Alt+Tab
    assert!(f.state.switcher.active());
    key(&mut f, 107, true);
    assert!(matches!(f.state.screenshot.phase, Phase::Requested));
    key(&mut f, 64, false);
    key(&mut f, 23, false);
    key(&mut f, 107, false);
    assert_eq!(f.state.focused_window(), Some(terminal.clone()));
    let generation = f.state.screenshot.generation;
    f.state.screenshot_ready(generation);
    assert!(!f.state.switcher.active());
    key(&mut f, 9, true);
    key(&mut f, 9, false);
    assert_eq!(f.state.focused_window(), Some(terminal));
}

#[test]
fn dragging_and_confirming_during_capture_preserves_the_requested_region() {
    let mut f = fixture();
    app(&mut f, "editor");
    key(&mut f, 107, true);
    f.state.screenshot_motion((25.0, 40.0).into());
    f.state.screenshot_button(0x110, ButtonState::Pressed);
    f.state.screenshot_motion((40.0, 60.0).into());
    f.state.screenshot_button(0x110, ButtonState::Released);
    key(&mut f, 36, true);
    assert!(matches!(f.state.screenshot.phase, Phase::Requested));
    let generation = f.state.screenshot.generation;
    f.state.screenshot_ready(generation);
    assert!(
        matches!(f.state.screenshot.phase,Phase::Exporting(r) if r==Rectangle::new((25,40).into(),(16,21).into()))
    );
    f.state.cancel_screenshot();
    f.state.screenshot_ready(generation);
    assert!(
        !f.state.screenshot.active(),
        "late capture result cannot reopen a cancelled selector"
    );
}

#[test]
fn cancelling_after_the_original_window_unmaps_restores_a_visible_window() {
    use smithay::reexports::wayland_server::Resource;
    let mut f = fixture();
    let fallback = app(&mut f, "browser");
    let original = app(&mut f, "terminal");
    let surface = original.toplevel().unwrap().wl_surface().id().protocol_id();
    key(&mut f, 107, true);
    let generation = f.state.screenshot.generation;
    f.state.screenshot_ready(generation);
    f.wire.request(surface, 1, &[0, 0, 0]);
    f.wire.request(surface, 6, &[]);
    f.dispatch();
    assert!(f.state.focused_window().is_none());
    f.state.cancel_screenshot();
    assert_eq!(f.state.focused_window(), Some(fallback));
}
