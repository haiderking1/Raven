use super::fixture::Fixture;
use smithay::{
    backend::input::ButtonState,
    input::pointer::{ButtonEvent, MotionEvent},
    reexports::wayland_server::Resource,
    utils::SERIAL_COUNTER,
};

#[test]
fn hover_changes_focus_once_and_preserves_background_and_drag_focus() {
    let mut f = Fixture::new();
    let first = f.toplevel();
    f.configure(first);
    let buffer = f.buffer();
    f.attach(first, buffer);
    let second = f.toplevel();
    f.configure(second);
    f.attach(second, buffer);
    let second_window = f.state.space.elements().next_back().unwrap().clone();
    f.state.space.map_element(second_window, (200, 0), false);
    let keyboard = f.state.seat.get_keyboard().unwrap();
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        second.surface
    );

    f.state.focus_window_on_motion((10.0, 10.0).into());
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        first.surface
    );
    let events = f.dispatch();
    assert!(
        events
            .iter()
            .any(|event| event.object == first.role && event.opcode == 0)
    );
    f.state.focus_window_on_motion((20.0, 20.0).into());
    assert!(
        f.dispatch().is_empty(),
        "motion within a focused window must not send more configures"
    );
    f.state.focus_window_on_motion((150.0, 20.0).into());
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        first.surface
    );
    assert!(f.dispatch().is_empty());

    // Use Smithay's real implicit button grab, as when selecting terminal text.
    let pointer = f.state.seat.get_pointer().unwrap();
    let location = (20.0, 20.0).into();
    let focus = f.state.surface_under(location);
    pointer.motion(
        &mut f.state,
        focus,
        &MotionEvent {
            location,
            serial: SERIAL_COUNTER.next_serial(),
            time: 1,
        },
    );
    pointer.button(
        &mut f.state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: 2,
            button: 0x110,
            state: ButtonState::Pressed,
        },
    );
    pointer.frame(&mut f.state);
    assert!(pointer.is_grabbed());
    f.state.focus_window_on_motion((210.0, 20.0).into());
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        first.surface
    );
    pointer.button(
        &mut f.state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: 3,
            button: 0x110,
            state: ButtonState::Released,
        },
    );
    pointer.frame(&mut f.state);
    assert!(!pointer.is_grabbed());
    f.state.focus_window_on_motion((210.0, 20.0).into());
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        second.surface
    );
}
