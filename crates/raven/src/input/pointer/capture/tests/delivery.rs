use super::{
    protocol::{Game, event, no_motion},
    wire::word,
};
use smithay::{
    backend::input::{Axis, AxisSource, ButtonState},
    input::pointer::{AxisFrame, ButtonEvent, MotionEvent, RelativeMotionEvent},
    reexports::wayland_server::Resource,
    utils::SERIAL_COUNTER,
};

pub(super) fn locked(game: &mut Game) {
    let pointer = game.f.state.seat.get_pointer().unwrap();
    let location = game.f.state.pointer_location;
    let relative = RelativeMotionEvent {
        delta: (240.5, -7.25).into(),
        delta_unaccel: (81.0, -3.5).into(),
        utime: 0x12_3456_789a,
    };
    let events = game.motion((260.5, 12.75), Some(relative));
    no_motion(&events, game.pointer);
    event(&events, game.pointer, 5);
    let args = &event(&events, game.relative, 0).args;
    let words: Vec<_> = args.chunks_exact(4).map(word).collect();
    assert_eq!(
        words,
        [
            0x12,
            0x3456_789a,
            (240.5 * 256.0) as i32 as u32,
            (-7.25 * 256.0) as i32 as u32,
            81 * 256,
            (-3.5 * 256.0) as i32 as u32
        ]
    );
    assert_eq!(game.f.state.pointer_location, location);
    assert_eq!(pointer.current_location(), location);
    assert_eq!(
        pointer.current_focus().unwrap().id().protocol_id(),
        game.top.surface
    );
    assert_eq!(
        game.f
            .state
            .seat
            .get_keyboard()
            .unwrap()
            .current_focus()
            .unwrap()
            .id()
            .protocol_id(),
        game.top.surface
    );

    assert!(
        !game
            .f
            .state
            .refresh_pointer_focus(SERIAL_COUNTER.next_serial(), 4)
    );
    // Bypass Raven's refresh filter to exercise the vendor's final grab-selected hook.
    let focus = game.f.state.surface_under(location);
    pointer.motion(
        &mut game.f.state,
        focus,
        &MotionEvent {
            location: (250.0, 20.0).into(),
            serial: SERIAL_COUNTER.next_serial(),
            time: 5,
        },
    );
    no_motion(&game.f.dispatch(), game.pointer);
    no_motion(&game.motion((250.0, 20.0), None), game.pointer);
    assert_eq!(pointer.current_location(), location);

    // Ordinary button grabs and scrolling remain on the locked game.
    for state in [ButtonState::Pressed, ButtonState::Released] {
        pointer.button(
            &mut game.f.state,
            &ButtonEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: 6,
                button: 0x110,
                state,
            },
        );
        pointer.frame(&mut game.f.state);
        let events = game.f.dispatch();
        event(&events, game.pointer, 3);
        no_motion(&events, game.pointer);
        assert!(game.f.state.pointer_is_locked());
    }
    pointer.axis(
        &mut game.f.state,
        AxisFrame::new(7)
            .source(AxisSource::Wheel)
            .value(Axis::Vertical, 15.0)
            .v120(Axis::Vertical, 120),
    );
    pointer.frame(&mut game.f.state);
    let events = game.f.dispatch();
    let args = &event(&events, game.pointer, 4).args;
    assert_eq!(
        [word(args), word(&args[4..]), word(&args[8..])],
        [7, 0, 15 * 256]
    );
    no_motion(&events, game.pointer);
}
