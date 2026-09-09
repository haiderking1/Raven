use super::super::layers::assertions::focus;
use super::{assertions::*, fixture::*};

#[test]
fn startup_last_request_waits_for_bufferless_commit_then_mapping() {
    for requests in [[true, false], [false, true]] {
        let mut f = fixture();
        let buffer = f.buffer_sized(800, 600);
        let (existing, incumbent) = map(&mut f, buffer);
        let top = f.toplevel();
        for fullscreen in requests {
            no_configure(&request(&mut f, top, fullscreen), top);
            owner(&f, None);
            focus(&f, existing.surface);
            assert_eq!(f.state.space().elements().count(), 1);
        }
        let fullscreen = requests[1];
        let size = if fullscreen { (800, 600) } else { (400, 600) };
        let serial = configured(&commit(&mut f, top), top, size, fullscreen);
        ack(&mut f, top, serial);
        owner(&f, None);
        commit(&mut f, top);
        owner(&f, None);
        assert_eq!(f.state.space().elements().count(), 1);
        let buffer = f.buffer_sized(size.0, size.1);
        f.attach(top, buffer);
        let mapped = window(&f, top);
        owner(&f, fullscreen.then_some(&mapped));
        layout(
            &f,
            &mapped,
            if fullscreen { (0, 0) } else { (400, 0) },
            (size.0 as i32, size.1 as i32),
        );
        assert_eq!(f.state.window_is_visible(&incumbent), !fullscreen);
        focus(&f, top.surface);
    }
}

#[test]
fn last_startup_mapping_replaces_owner_without_removing_tiles() {
    let mut f = fixture();
    let buffer = f.buffer_sized(800, 600);
    let (first_top, first) = startup(&mut f, buffer);
    let (top, second, events) = startup_with_events(&mut f, buffer);
    let displaced = latest_configured(&events, first_top, (400, 600), false);
    // The displaced client still has fullscreen in its committed xdg state.
    commit(&mut f, first_top);
    owner(&f, Some(&second));
    assert!(!f.state.window_is_visible(&first));
    assert!(f.state.window_is_visible(&second));
    focus(&f, top.surface);
    // Exit the replacement before the displaced client acknowledges its exit.
    // Its old fullscreen-sized buffer must already be confined to its tile.
    let serial = configured(&request(&mut f, top, false), top, (400, 600), false);
    ack(&mut f, top, serial);
    commit(&mut f, top);
    owner(&f, None);
    layout(&f, &first, (0, 0), (400, 600));
    layout(&f, &second, (400, 0), (400, 600));
    assert!(f.state.window_is_visible(&first));
    ack(&mut f, first_top, displaced);
    commit(&mut f, first_top);
    owner(&f, None);
    layout(&f, &first, (0, 0), (400, 600));
}
