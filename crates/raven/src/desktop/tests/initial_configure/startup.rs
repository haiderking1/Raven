use super::super::wire::word;
use super::{assertions::tiled, fixture::fixture};
use smithay::reexports::wayland_server::Resource;

#[test]
fn startup_mode_requests_wait_for_the_initial_commit_and_include_tile_geometry() {
    for occupied in [false, true] {
        // A saved maximized/fullscreen preference may arrive before the first commit.
        for (opcode, args) in [(9, vec![]), (11, vec![0])] {
            let mut f = fixture(occupied);
            let top = f.toplevel();
            f.wire.request(top.role, opcode, &args);
            let events = f.dispatch();
            assert!(
                !events
                    .iter()
                    .any(|e| e.object == top.role || e.object == top.xdg),
                "role request sent a configure before initial commit: {events:?}"
            );
            assert_eq!(f.state.space().elements().count(), usize::from(occupied));

            f.wire.request(top.surface, 6, &[]);
            let events = f.dispatch();
            let size = (if occupied { 501 } else { 1001 }, 601);
            let serial = tiled(&events, top, size);
            let bounds = events
                .iter()
                .find(|e| e.object == top.role && e.opcode == 2)
                .unwrap();
            assert_eq!((word(&bounds.args), word(&bounds.args[4..])), size);
            f.wire.request(top.xdg, 4, &[serial]);
            // Behave like a client that draws exactly the size Raven requested.
            let buffer = f.buffer_sized(size.0, size.1);
            f.wire.request(top.surface, 1, &[buffer, 0, 0]);
            f.wire.request(top.surface, 6, &[]);
            let events = f.dispatch();
            let window = f
                .state
                .space()
                .elements()
                .find(|window| {
                    window.toplevel().unwrap().wl_surface().id().protocol_id() == top.surface
                })
                .unwrap();
            assert_eq!(
                window.geometry().size,
                (size.0 as i32, size.1 as i32).into()
            );
            assert_eq!(
                f.state.space().element_location(window),
                Some((if occupied { 500 } else { 0 }, 0).into())
            );
            // Activation can send a configure, but mapping must not repair its size.
            for event in events
                .iter()
                .filter(|e| e.object == top.role && e.opcode == 0)
            {
                assert_eq!((word(&event.args), word(&event.args[4..])), size);
            }
        }
    }
}
