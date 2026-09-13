use crate::{
    state::Server,
    wire::{Event, word},
};

fn modes(events: &[Event], object: u32) -> Vec<u32> {
    events
        .iter()
        .filter(|e| e.object == object && e.opcode == 0)
        .map(|e| word(&e.args))
        .collect()
}

#[test]
fn advertises_server_mode_and_overrides_all_client_preferences() {
    let mut server = Server::new();
    assert_eq!(modes(&server.dispatch(), 4), [2]);
    server.wire.request(3, 0, &[5]); // wl_surface
    server.wire.request(4, 0, &[6, 5]); // decoration for that surface
    assert_eq!(modes(&server.dispatch(), 6), [2]);
    for preference in [0, 1, 2, 999] {
        // None, Client, Server, unknown
        server.wire.request(6, 1, &[preference]);
        assert_eq!(modes(&server.dispatch(), 6), [2]);
    }
}

#[test]
fn bounds_echo_replies_per_object_and_resets_on_recreation() {
    let mut server = Server::new();
    server.dispatch();
    server.wire.request(3, 0, &[5]);
    server.wire.request(4, 0, &[6, 5]);
    server.dispatch();
    for _ in 0..8 {
        server.wire.request(6, 1, &[1]);
    }
    assert_eq!(modes(&server.dispatch(), 6), [2, 2, 2, 2]);

    // One exhausted object must not suppress a different object's replies.
    server.wire.request(3, 0, &[7]);
    server.wire.request(4, 0, &[8, 7]);
    server.wire.request(8, 1, &[1]);
    assert_eq!(modes(&server.dispatch(), 8), [2, 2]);

    server.wire.request(6, 0, &[]); // release
    server.wire.request(4, 0, &[9, 5]); // same surface, fresh decoration
    server.wire.request(9, 1, &[1]);
    assert_eq!(modes(&server.dispatch(), 9), [2, 2]);

    // Surface destruction before decoration release must remain harmless.
    server.wire.request(5, 0, &[]);
    server.wire.request(9, 0, &[]);
    server.wire.request(8, 1, &[2]);
    assert_eq!(modes(&server.dispatch(), 8), [2]);
}
