use super::{
    protocol::{Game, event, no_motion},
    wire::word,
};
use smithay::{
    input::pointer::MotionEvent, reexports::wayland_server::Resource, utils::SERIAL_COUNTER,
};

pub(super) fn exercise(game: &mut Game) {
    let lock = game.constraint(true, 0, 2);
    event(&game.f.dispatch(), lock, 0);
    game.f.wire.request(lock, 1, &[70 * 256, 70 * 256]);
    game.commit();
    let before = game.f.state.pointer_location;
    let keyboard = game.f.state.seat.get_keyboard().unwrap();
    let neighbor = game
        .f
        .state
        .space()
        .elements()
        .find(|w| w.toplevel().unwrap().wl_surface().id().protocol_id() == game.neighbor.surface)
        .unwrap()
        .toplevel()
        .unwrap()
        .wl_surface()
        .clone();
    keyboard.set_focus(
        &mut game.f.state,
        Some(neighbor),
        SERIAL_COUNTER.next_serial(),
    );
    let events = game.f.dispatch();
    event(&events, lock, 1);
    no_motion(&events, game.pointer);
    assert_eq!(
        game.f.state.pointer_location, before,
        "forced focus loss must ignore hints"
    );
    game.motion((210.0, 20.0), None);
    event(&game.motion((20.0, 20.0), None), lock, 0);

    game.f.state.switch_workspace(1);
    event(&game.f.dispatch(), lock, 1);
    assert!(!game.f.state.pointer_is_captured());
    game.f.state.switch_workspace(0);
    event(&game.f.dispatch(), lock, 0);
    game.f.state.suspend_pointer_capture();
    event(&game.f.dispatch(), lock, 1);
    game.f
        .state
        .refresh_pointer_focus(SERIAL_COUNTER.next_serial(), 10);
    assert!(!game.f.state.pointer_is_captured());
    game.f.state.resume_pointer_capture();
    game.f
        .state
        .refresh_pointer_focus(SERIAL_COUNTER.next_serial(), 11);
    event(&game.f.dispatch(), lock, 0);
    game.f.wire.request(lock, 0, &[]);
    game.f.dispatch();
    game.motion((20.0, 20.0), None);

    let oneshot = game.constraint(true, 0, 1);
    event(&game.f.dispatch(), oneshot, 0);
    let pointer = game.f.state.seat.get_pointer().unwrap();
    pointer.motion(
        &mut game.f.state,
        None,
        &MotionEvent {
            location: (20.0, 20.0).into(),
            serial: SERIAL_COUNTER.next_serial(),
            time: 12,
        },
    );
    event(&game.f.dispatch(), oneshot, 1);
    let replacement = game.constraint(true, 0, 2);
    let empty = game.f.id();
    game.f.wire.request(3, 1, &[empty]);
    game.f.wire.request(oneshot, 2, &[empty]);
    game.f.dispatch();
    game.f.wire.request(oneshot, 0, &[]);
    let events = game.f.dispatch();
    assert!(!events.iter().any(|e| e.object == replacement));
    event(&game.motion((20.0, 20.0), None), replacement, 0);
    game.commit();
    assert!(
        game.f.state.pointer_is_locked(),
        "old oneshot requests/destroy must not touch its replacement"
    );
    game.f.wire.request(replacement, 0, &[]);
    let events = game.f.dispatch();
    assert!(
        !events
            .iter()
            .any(|e| e.object == game.pointer && e.opcode == 2),
        "no hint means no warp: {events:?}"
    );
    assert!(!game.f.state.pointer_is_captured());
    // Server delete_id events also prove that requests reached actual resources.
    assert!(
        events
            .iter()
            .any(|e| e.object == 1 && e.opcode == 1 && word(&e.args) == replacement)
    );
}
