use crate::desktop::tests::{
    fixture::{Fixture, Toplevel},
    wire::string,
};
use smithay::{
    desktop::Window,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_server::Resource,
};
fn fixture() -> Fixture {
    let mut f = Fixture::new();
    let output = Output::new(
        "switcher-test".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (1280, 800).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    f.state.space_mut().map_output(&output, (0, 0));
    f.state.output = Some(output);
    f
}
fn app(f: &mut Fixture, id: &str) -> (Toplevel, Window) {
    let top = f.toplevel();
    f.wire.bytes(top.role, 3, &string(id), None);
    f.configure(top);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let window = f
        .state
        .windows
        .iter()
        .find(|w| {
            w.toplevel()
                .is_some_and(|t| t.wl_surface().id().protocol_id() == top.surface)
        })
        .unwrap()
        .clone();
    f.state.activate_window(Some(window.clone()));
    (top, window)
}
#[test]
fn preview_does_not_change_focus_and_cancel_restores_no_other_window() {
    let mut f = fixture();
    let (_, first) = app(&mut f, "test-browser");
    let (_, terminal) = app(&mut f, "test-terminal");
    let (_, recent) = app(&mut f, "test-browser");
    f.state.cycle_applications(false);
    assert_eq!(f.state.focused_window(), Some(recent.clone()));
    assert_eq!(
        f.state.switcher.session.as_ref().unwrap().target(),
        Some(&terminal)
    );
    f.state.cancel_app_switcher();
    assert_eq!(f.state.focused_window(), Some(recent.clone()));
    f.state.cycle_applications(false);
    f.state.confirm_app_switcher();
    assert_eq!(f.state.focused_window(), Some(terminal));
    f.state.cycle_applications(false);
    f.state.confirm_app_switcher();
    assert_eq!(f.state.focused_window(), Some(recent));
    assert_ne!(f.state.focused_window(), Some(first));
}
#[test]
fn confirming_an_app_changes_workspace_only_after_preview() {
    let mut f = fixture();
    let (_, browser) = app(&mut f, "test-browser");
    f.state.switch_workspace(1);
    let (_, terminal) = app(&mut f, "test-terminal");
    f.state.cycle_applications(false);
    assert_eq!(f.state.workspaces.active, 1);
    assert_eq!(f.state.focused_window(), Some(terminal.clone()));
    f.state.confirm_app_switcher();
    assert_eq!(f.state.workspaces.active, 0);
    assert_eq!(f.state.focused_window(), Some(browser));
    f.state.cycle_applications(false);
    f.state.confirm_app_switcher();
    assert_eq!(f.state.workspaces.active, 1);
    assert_eq!(f.state.focused_window(), Some(terminal));
}

#[test]
fn alt_release_confirms_after_xkb_updates_and_late_tab_release_is_swallowed() {
    use smithay::{
        backend::input::KeyState::{Pressed, Released},
        input::keyboard::Keycode,
    };
    let mut f = fixture();
    let (_, browser) = app(&mut f, "test-browser");
    let (_, terminal) = app(&mut f, "test-terminal");
    let key = |f: &mut Fixture, code, state| {
        crate::input::keyboard::dispatch(&mut f.state, Keycode::new(code), state, 100)
    };
    key(&mut f, 64, Pressed); // left Alt
    key(&mut f, 23, Pressed); // Tab
    assert!(f.state.switcher.active());
    assert_eq!(f.state.focused_window(), Some(terminal));
    key(&mut f, 64, Released);
    assert!(!f.state.switcher.active());
    assert_eq!(f.state.focused_window(), Some(browser.clone()));
    key(&mut f, 23, Released);
    assert_eq!(f.state.focused_window(), Some(browser));
}
