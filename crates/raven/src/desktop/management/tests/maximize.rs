use super::fixture::Harness;
use crate::desktop::{floating::maximize, management::minimize};
#[test]
fn taskbar_maximize_restores_tiled_and_floating_placement() {
    let mut h = Harness::new(3);
    let (top, window, handle) = h.app("zoom");
    let (_, _, _) = h.app("other");
    let original = h.f.state.window_frame_geometry(&window).unwrap();
    h.events.clear();
    h.f.wire.request(top.role, 9, &[]);
    h.settle();
    assert_eq!(
        h.events
            .iter()
            .filter(|e| e.object == top.role && e.opcode == 0)
            .count(),
        1
    );
    assert!(maximize::is_maximized(&window));
    assert_eq!(
        h.f.state.window_frame_geometry(&window),
        h.f.state.window_workarea()
    );
    assert!(h.states(handle).contains(&0));
    h.request(handle, 1);
    assert!(!maximize::is_maximized(&window));
    assert_eq!(h.f.state.window_frame_geometry(&window), Some(original));
    h.f.state.activate_window(Some(window.clone()));
    h.f.state.toggle_focused_floating();
    h.settle();
    let floating = h.f.state.window_frame_geometry(&window).unwrap();
    h.request(handle, 0);
    h.request(handle, 1);
    assert!(h.f.state.window_is_floating(&window));
    assert_eq!(h.f.state.window_frame_geometry(&window), Some(floating));
    h.request(handle, 0);
    h.f.wire.request(top.role, 7, &[400, 300]);
    h.f.wire.request(top.surface, 6, &[]);
    h.settle();
    assert_eq!(
        h.f.state.window_client_geometry(&window).unwrap().size,
        (400, 300).into()
    );
    h.f.wire.request(top.role, 7, &[0, 0]);
    h.f.wire.request(top.surface, 6, &[]);
    h.settle();
    assert_eq!(
        h.f.state.window_frame_geometry(&window),
        h.f.state.window_workarea()
    );
}

#[test]
fn initial_maximize_and_legacy_taskbar_restore_use_real_configures() {
    let mut h = Harness::new(1);
    let (top, window, handle) = h.app_options("initial-max", None, true);
    assert!(maximize::is_maximized(&window));
    assert_eq!(
        h.f.state.window_frame_geometry(&window),
        h.f.state.window_workarea()
    );
    h.f.wire.request(top.role, 10, &[]);
    h.settle();
    assert!(!maximize::is_maximized(&window));
    assert!(h.f.state.window_is_tiled(&window));
    h.request(handle, 2);
    assert!(minimize::hidden(&window));
    h.request(handle, 3);
    assert!(!minimize::hidden(&window));
}

#[test]
fn fullscreen_minimize_waits_for_exit_and_manual_move_releases_maximize() {
    let mut h = Harness::new(3);
    let (top, window, handle) = h.app("full");
    h.f.wire.request(top.role, 11, &[0]);
    h.settle();
    assert!(h.f.state.applied_fullscreen_allocation(&window).is_some());
    h.request(handle, 2);
    assert!(minimize::hidden(&window));
    assert!(h.f.state.applied_fullscreen_allocation(&window).is_none());
    h.request(handle, 3);
    h.request(handle, 0);
    h.f.state.move_floating_window(&window, (20, 30).into());
    h.settle();
    assert!(!maximize::is_maximized(&window));
    assert_eq!(
        h.f.state.window_frame_geometry(&window).unwrap().loc,
        (20, 30).into()
    );
}
