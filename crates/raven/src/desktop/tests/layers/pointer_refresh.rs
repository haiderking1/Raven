use super::super::wire::word;
use super::{fixture::fixture, pointer_wire::bind_pointer, protocol::LayerClient};
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1::{
    Anchor, KeyboardInteractivity,
};

#[test]
fn redraws_do_not_move_the_pointer_but_geometry_and_input_region_changes_do() {
    let (mut f, shell) = fixture();
    let pointer = bind_pointer(&mut f);
    f.state.pointer_location = (360.0, 260.0).into();
    let buffer = f.buffer();
    let launcher = LayerClient::new(&mut f, shell);
    launcher.settings(&mut f, Anchor::empty(), 0, KeyboardInteractivity::Exclusive);
    let serial = launcher.configure(&mut f);
    launcher.ack(&mut f, serial);
    let events = launcher.attach(&mut f, buffer);
    assert!(
        events
            .iter()
            .any(|event| event.object == pointer && event.opcode == 0)
    );

    // Fuzzel redraws after arrow-key selection. That must not act like mouse input.
    let events = launcher.attach(&mut f, buffer);
    assert!(
        !events.iter().any(|event| event.object == pointer),
        "redraw emitted pointer events: {events:?}"
    );

    // Actual mouse movement still reaches the client, without a redraw echo.
    f.state.pointer_location = (365.0, 260.0).into();
    let focus = f.state.surface_under(f.state.pointer_location);
    f.state.send_pointer_motion(
        focus,
        &smithay::input::pointer::MotionEvent {
            location: f.state.pointer_location,
            serial: smithay::utils::SERIAL_COUNTER.next_serial(),
            time: 1,
        },
    );
    f.state.seat.get_pointer().unwrap().frame(&mut f.state);
    let events = f.dispatch();
    assert!(
        events
            .iter()
            .any(|event| event.object == pointer && event.opcode == 2)
    );
    let events = launcher.attach(&mut f, buffer);
    assert!(!events.iter().any(|event| event.object == pointer));

    // The root moves left by ten pixels but still lies under the stationary mouse.
    f.wire.request(launcher.role, 0, &[120, 100]);
    let events = launcher.commit(&mut f);
    let motions: Vec<_> = events
        .iter()
        .filter(|event| event.object == pointer && event.opcode == 2)
        .collect();
    assert_eq!(motions.len(), 1);
    assert_eq!(
        (word(&motions[0].args[4..]), word(&motions[0].args[8..])),
        (25 * 256, 10 * 256)
    );

    // Input-region changes must still send leave, even without a buffer resize.
    let region = f.id();
    f.wire.request(3, 1, &[region]);
    f.wire.request(launcher.surface, 5, &[region]);
    let events = launcher.commit(&mut f);
    assert!(
        events
            .iter()
            .any(|event| event.object == pointer && event.opcode == 1)
    );
    f.wire.request(launcher.surface, 5, &[0]);
    let events = launcher.commit(&mut f);
    assert!(
        events
            .iter()
            .any(|event| event.object == pointer && event.opcode == 0)
    );
}
