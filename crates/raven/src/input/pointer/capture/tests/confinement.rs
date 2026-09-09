use super::{
    protocol::{Game, event},
    wire::word,
};
use smithay::{input::pointer::RelativeMotionEvent, reexports::wayland_server::Resource};

pub(super) fn exercise(game: &mut Game) {
    let region = game.region(10, 80, true);
    let confined = game.constraint(false, region, 2);
    event(&game.f.dispatch(), confined, 0);
    let relative = || RelativeMotionEvent {
        delta: (60.0, 0.0).into(),
        delta_unaccel: (30.0, 0.0).into(),
        utime: 123_456,
    };
    let events = game.motion((80.0, 20.0), Some(relative()));
    let x = game.f.state.pointer_location.x;
    assert!(
        x > 39.0 && x < 40.0,
        "must stop before the hole, not jump across: {x}"
    );
    event(&events, game.pointer, 2);
    let args = &event(&events, game.relative, 0).args;
    assert_eq!(
        word(&args[8..]),
        60 * 256,
        "clipping must not change relative deltas"
    );
    let full = game.region(0, 100, false);
    game.f.wire.request(confined, 1, &[full]);
    game.f.dispatch();
    game.motion((80.0, 20.0), Some(relative()));
    assert!(
        game.f.state.pointer_location.x < 40.0,
        "set_region is pending until surface commit"
    );
    game.commit();
    game.motion((80.0, 20.0), Some(relative()));
    assert_eq!(game.f.state.pointer_location, (80.0, 20.0).into());
    game.motion((250.0, 20.0), Some(relative()));
    assert!(game.f.state.pointer_location.x < 100.0);
    assert_eq!(
        game.f
            .state
            .seat
            .get_pointer()
            .unwrap()
            .current_focus()
            .unwrap()
            .id()
            .protocol_id(),
        game.top.surface
    );

    // Current wl_surface input region intersects the explicit constraint.
    let input = game.region(0, 35, false);
    game.f.wire.request(game.top.surface, 5, &[input]);
    event(&game.commit(), confined, 1);
    assert!(!game.f.state.pointer_is_captured());
    event(&game.motion((20.0, 20.0), None), confined, 0);
    game.motion((80.0, 20.0), Some(relative()));
    assert!(game.f.state.pointer_location.x < 35.0);

    // Null-buffer unmap must release before any later physical motion arrives.
    game.f.wire.request(game.top.surface, 1, &[0, 0, 0]);
    event(&game.commit(), confined, 1);
    assert!(!game.f.state.pointer_is_captured());
    game.f.configure(game.top);
    let buffer = game.f.buffer();
    game.f.attach(game.top, buffer);
    assert!(
        game.f.state.pointer_is_captured(),
        "persistent confinement reactivates on remap"
    );
    game.f.wire.request(game.top.surface, 0, &[]);
    event(&game.f.dispatch(), confined, 1);
    assert!(!game.f.state.pointer_is_captured());
    game.f.wire.request(confined, 0, &[]);
    game.f.dispatch();
}
