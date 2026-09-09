use super::super::layers::assertions::focus;
use super::{assertions::*, fixture::*};
use smithay::{
    backend::input::ButtonState,
    input::pointer::{ButtonEvent, MotionEvent},
    utils::SERIAL_COUNTER,
};

#[test]
fn null_unmap_clears_current_and_pending_intent_before_remap() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (other, survivor) = map(&mut f, buffer);
    let (top, window) = map(&mut f, buffer);
    enter(&mut f, top);
    owner(&f, Some(&window));
    // Smithay can retain fullscreen in current state across this null commit.
    f.attach(top, 0);
    owner(&f, None);
    assert!(!f.state.window_is_visible(&window));
    assert!(f.state.window_is_visible(&survivor));
    focus(&f, other.surface);
    let serial = configured(&commit(&mut f, top), top, (400, 600), false);
    ack(&mut f, top, serial);
    f.attach(top, buffer);
    owner(&f, None);
    layout(&f, &window, (400, 0), (400, 600));

    let serial = configured(&request(&mut f, top, true), top, (800, 600), true);
    ack(&mut f, top, serial);
    f.attach(top, 0);
    owner(&f, None);
    let serial = configured(&commit(&mut f, top), top, (400, 600), false);
    ack(&mut f, top, serial);
    f.attach(top, buffer);
    owner(&f, None);
    assert!(f.state.window_is_visible(&survivor));
}

#[test]
fn destroy_and_disconnect_remove_fullscreen_ownership() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (other, survivor) = map(&mut f, buffer);
    let (top, window) = startup(&mut f, buffer);
    destroy(&mut f, top);
    f.state.refresh();
    owner(&f, None);
    assert!(!f.state.windows.contains(&window));
    assert_eq!(f.state.workspaces.index_of(&window), None);
    assert!(!f.state.window_is_visible(&window));
    layout(&f, &survivor, (0, 0), (800, 600));
    focus(&f, other.surface);
    enter(&mut f, other);
    owner(&f, Some(&survivor));
    f.disconnect();
    f.state.refresh();
    owner(&f, None);
    assert!(f.state.windows.is_empty());
    assert_eq!(f.state.workspaces.index_of(&survivor), None);
    assert!(!f.state.window_is_visible(&survivor));
}

#[test]
fn a_new_fullscreen_claim_waits_for_the_hidden_clients_button_release() {
    let mut f = fixture();
    let buffer = f.buffer_sized(800, 600);
    let (first, first_window) = map(&mut f, buffer);
    let pointer = f.state.seat.get_pointer().unwrap();
    let location = (20.0, 20.0).into();
    let pointer_focus = f.state.surface_under(location);
    pointer.motion(
        &mut f.state,
        pointer_focus,
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
    assert!(pointer.is_grabbed());
    let (second, second_window) = startup(&mut f, buffer);
    f.state.refresh();
    owner(&f, None);
    focus(&f, first.surface);
    assert!(f.state.window_is_visible(&first_window));
    assert!(pointer.is_grabbed());
    pointer.button(
        &mut f.state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: 3,
            button: 0x110,
            state: ButtonState::Released,
        },
    );
    assert!(!pointer.is_grabbed());
    f.state.refresh();
    owner(&f, Some(&second_window));
    focus(&f, second.surface);
    assert!(!f.state.window_is_visible(&first_window));
}

#[test]
fn output_geometry_is_committed_and_output_loss_cancels_fullscreen_intent() {
    use smithay::output::Mode;
    let mut f = fixture();
    let buffer = f.buffer_sized(400, 600);
    let (top, window) = startup(&mut f, buffer);
    let output = f.state.output.clone().unwrap();
    output.change_current_state(
        Some(Mode {
            size: (1001, 701).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((13, 17).into()),
    );
    f.state.space_mut().map_output(&output, (13, 17));
    f.state.refresh();
    let serial = latest_configured(&f.dispatch(), top, (1001, 701), true);
    layout(&f, &window, (0, 0), (800, 600));
    ack(&mut f, top, serial);
    layout(&f, &window, (0, 0), (800, 600));
    commit(&mut f, top);
    layout(&f, &window, (13, 17), (1001, 701));
    assert_eq!(
        f.state.space().element_location(&window),
        Some((313, 67).into())
    );
    f.state.output = None;
    f.state.refresh();
    owner(&f, None);
    f.dispatch();
    f.state.output = Some(output);
    f.state.refresh();
    owner(&f, None);
    let serial = latest_configured(&f.dispatch(), top, (1001, 701), false);
    ack(&mut f, top, serial);
    commit(&mut f, top);
    owner(&f, None);
    layout(&f, &window, (13, 17), (1001, 701));
    assert_eq!(
        f.state.space().element_location(&window),
        Some((13, 17).into())
    );
}
