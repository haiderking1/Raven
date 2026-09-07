use super::fixture::Fixture;
use smithay::input::{SeatHandler, pointer::CursorImageStatus};

#[test]
fn commits_and_server_side_changes_request_redraw_without_idle_polling() {
    let mut f = Fixture::new();
    let top = f.toplevel();
    f.configure(top);
    let buffer = f.buffer();
    f.attach(top, buffer);
    f.state.refresh();
    assert!(f.state.take_redraw_request());
    f.state.refresh();
    assert!(
        !f.state.take_redraw_request(),
        "idle reconciliation must not request rendering"
    );

    // A callback-only commit must wake the backend even without new pixel damage.
    let callback = f.id();
    f.wire.request(top.surface, 3, &[callback]);
    f.wire.request(top.surface, 6, &[]);
    f.dispatch();
    assert!(f.state.take_redraw_request());
    assert!(
        !f.state.take_redraw_request(),
        "requests are consumed once per batch"
    );

    // Role-only destruction must erase a subsurface even if its wl_surface lives.
    let subcompositor = super::globals::bind(&mut f, "wl_subcompositor", 1);
    let child = f.id();
    let role = f.id();
    f.wire.request(3, 0, &[child]);
    f.wire
        .request(subcompositor, 1, &[role, child, top.surface]);
    f.wire.request(child, 1, &[buffer, 0, 0]);
    f.wire.request(child, 6, &[]);
    f.wire.request(top.surface, 6, &[]);
    f.dispatch();
    assert!(f.state.take_redraw_request());
    f.wire.request(role, 0, &[]);
    f.dispatch();
    assert!(
        f.state.take_redraw_request(),
        "subsurface role removal must invalidate the scene"
    );

    // These change the scene without requiring a client buffer commit.
    let seat = f.state.seat.clone();
    f.state.cursor_image(&seat, CursorImageStatus::Hidden);
    assert!(f.state.take_redraw_request());
    f.state.switch_workspace(1);
    assert!(f.state.take_redraw_request());
    f.state.refresh();
    assert!(!f.state.take_redraw_request());
    f.wire.request(top.surface, 1, &[buffer, 0, 0]);
    f.wire.request(top.surface, 6, &[]);
    f.dispatch();
    assert!(
        !f.state.take_redraw_request(),
        "hidden content must not repaint the active output"
    );
    f.state.switch_workspace(0);
    assert!(f.state.take_redraw_request());
    f.wire.request(top.role, 0, &[]);
    f.dispatch();
    assert_eq!(f.state.space().elements().count(), 0);
    assert!(
        f.state.take_redraw_request(),
        "role destruction must erase its window"
    );
}
