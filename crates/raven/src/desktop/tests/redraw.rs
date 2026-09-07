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

    // These change the scene without requiring a client buffer commit.
    let seat = f.state.seat.clone();
    f.state.cursor_image(&seat, CursorImageStatus::Hidden);
    assert!(f.state.take_redraw_request());
    f.state.switch_workspace(1);
    assert!(f.state.take_redraw_request());
    f.state.refresh();
    assert!(!f.state.take_redraw_request());
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
