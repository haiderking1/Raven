mod confinement;
mod delivery;
#[allow(dead_code)]
#[path = "../../../../desktop/tests/fixture.rs"]
mod fixture;
mod lifetime;
mod protocol;
#[path = "../../../../desktop/tests/wire.rs"]
mod wire;

use protocol::{Game, event, no_motion};
use wire::word;

#[test]
fn real_wire_game_capture_delivery_and_lifecycle() {
    let mut game = Game::new();
    game.motion((5.0, 20.0), None);
    let region = game.region(10, 80, false);
    let lock = game.constraint(true, region, 2);
    assert!(!game.f.dispatch().iter().any(|e| e.object == lock));
    assert!(
        !game.f.state.pointer_is_captured(),
        "outside the activation region"
    );
    let events = game.motion((20.0, 20.0), None);
    event(&events, lock, 0);
    delivery::locked(&mut game);

    // Only a committed hint may restore the cursor. A later pending hint must not win.
    game.f.wire.request(lock, 1, &[30 * 256, 35 * 256]);
    no_motion(&game.commit(), game.pointer);
    game.f.wire.request(lock, 1, &[80 * 256, 80 * 256]);
    game.f.wire.request(lock, 0, &[]);
    let events = game.f.dispatch();
    let args = &event(&events, game.pointer, 2).args;
    assert_eq!([word(&args[4..]), word(&args[8..])], [30 * 256, 35 * 256]);
    assert_eq!(game.f.state.pointer_location, (30.0, 35.0).into());
    assert!(!game.f.state.pointer_is_captured());

    lifetime::exercise(&mut game);
    confinement::exercise(&mut game);
}
