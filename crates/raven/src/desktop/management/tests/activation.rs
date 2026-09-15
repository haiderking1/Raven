use super::fixture::Harness;
use crate::desktop::management::minimize;
#[test]
fn taskbar_activation_restores_the_workspace_and_waits_for_fullscreen_exit() {
    let mut h = Harness::new(3);
    let (first_top, first, _) = h.app("first-activation");
    let (_, target, handle) = h.app("target-activation");
    let seat = h.seat();
    h.f.state.activate_window(Some(target.clone()));
    h.f.state.move_focused_to_workspace(1);
    h.settle();
    h.request(handle, 2);
    assert!(minimize::hidden(&target));
    h.f.wire.request(handle, 4, &[seat]);
    h.settle();
    assert_eq!(h.f.state.workspaces.active, 1);
    assert_eq!(h.f.state.focused_window(), Some(target.clone()));
    h.f.state.switch_workspace(0);
    h.f.state.activate_window(Some(first.clone()));
    h.f.state.move_focused_to_workspace(1);
    h.settle();
    h.f.state.switch_workspace(1);
    assert_eq!(h.f.state.workspaces.index_of(&first), Some(1));
    assert!(h.f.state.window_is_visible(&first));
    h.f.state.activate_window(Some(first.clone()));
    h.f.wire.request(first_top.role, 11, &[0]);
    h.settle();
    assert_eq!(h.f.state.fullscreen_window(), Some(&first));
    h.f.wire.request(handle, 4, &[seat]);
    h.settle();
    assert!(h.f.state.fullscreen_window().is_none());
    assert_eq!(h.f.state.focused_window(), Some(target.clone()));
    h.f.state.activate_window(Some(first.clone()));
    h.f.wire.request(first_top.role, 11, &[0]);
    h.settle();
    h.f.wire.request(handle, 8, &[0]);
    h.settle();
    assert_eq!(h.f.state.fullscreen_window(), Some(&target));
    h.request(handle, 9);
    assert!(h.f.state.fullscreen_window().is_none());
}

#[test]
fn taskbar_activation_waits_for_the_original_click_release() {
    use smithay::{
        backend::input::ButtonState,
        input::pointer::{ButtonEvent, MotionEvent},
        utils::SERIAL_COUNTER,
    };
    let mut h = Harness::new(3);
    let (_, first, _) = h.app("pressed");
    let (_, target, handle) = h.app("clicked-target");
    let seat = h.seat();
    h.f.state.activate_window(Some(first.clone()));
    let pointer = h.f.state.seat.get_pointer().unwrap();
    let serial = SERIAL_COUNTER.next_serial();
    pointer.motion(
        &mut h.f.state,
        Some((
            first.toplevel().unwrap().wl_surface().clone(),
            (0.0, 0.0).into(),
        )),
        &MotionEvent {
            location: (100.0, 100.0).into(),
            serial,
            time: 1,
        },
    );
    pointer.button(
        &mut h.f.state,
        &ButtonEvent {
            serial,
            time: 2,
            button: 0x110,
            state: ButtonState::Pressed,
        },
    );
    assert!(pointer.is_grabbed());
    h.f.wire.request(handle, 4, &[seat]);
    h.settle();
    assert_eq!(h.f.state.focused_window(), Some(first));
    assert!(pointer.is_grabbed());
    pointer.button(
        &mut h.f.state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: 3,
            button: 0x110,
            state: ButtonState::Released,
        },
    );
    h.settle();
    assert!(!pointer.is_grabbed());
    assert_eq!(h.f.state.focused_window(), Some(target));
}
