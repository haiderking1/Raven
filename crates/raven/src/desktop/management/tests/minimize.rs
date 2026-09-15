use super::fixture::Harness;
use crate::desktop::management::minimize;
#[test]
fn taskbar_minimize_restore_and_handle_lifetime_preserve_focus_and_tile_order() {
    let mut h = Harness::new(3);
    let (_, first, a) = h.app("first");
    let (second_top, second, b) = h.app("second");
    let original = (
        h.f.state.window_frame_geometry(&first),
        h.f.state.window_frame_geometry(&second),
    );
    h.f.state.activate_window(Some(first.clone()));
    h.request(a, 2);
    assert!(minimize::hidden(&first));
    assert!(!h.f.state.window_is_visible(&first));
    assert_eq!(h.f.state.focused_window(), Some(second.clone()));
    assert!(h.states(a).contains(&1));
    assert_eq!(h.f.state.workspaces.entries[0].space.elements().count(), 2);
    h.f.state.cycle_applications(false);
    h.f.state.confirm_app_switcher();
    assert!(minimize::hidden(&first));
    assert!(h.f.state.switcher.pending.is_none());
    assert_eq!(h.f.state.focused_window(), Some(second.clone()));
    h.request(a, 0);
    assert!(minimize::hidden(&first));
    h.request(a, 1);
    assert!(minimize::hidden(&first));
    h.request(a, 3);
    assert!(!minimize::hidden(&first));
    assert_eq!(
        (
            h.f.state.window_frame_geometry(&first),
            h.f.state.window_frame_geometry(&second)
        ),
        original
    );
    assert_eq!(h.f.state.focused_window(), Some(second.clone()));
    h.request(a, 2);
    h.request(h.manager, 0);
    assert!(
        minimize::hidden(&first),
        "stopped managers retain existing handles"
    );
    h.request(b, 7);
    h.f.wire.request(second_top.role, 13, &[]);
    h.settle();
    assert!(
        !minimize::hidden(&second),
        "another window's handle is not a restore route"
    );
    h.request(a, 7);
    assert!(
        !minimize::hidden(&first),
        "losing the final restore route must reveal hidden windows"
    );
}

#[test]
fn minimizing_a_parent_hides_its_dialog_and_unmap_closes_only_its_handle() {
    let mut h = Harness::new(3);
    let (top, parent, handle) = h.app("parent");
    let (_, dialog, child) = h.app_options("dialog", Some(top), true);
    h.request(child, 1);
    assert!(h.f.state.window_is_floating(&dialog));
    h.request(handle, 2);
    assert!(minimize::hidden(&parent));
    assert!(minimize::hidden(&dialog));
    assert!(!h.f.state.window_is_visible(&dialog));
    h.request(child, 0);
    h.request(child, 1);
    h.request(handle, 3);
    assert!(!minimize::hidden(&dialog));
    h.f.wire.request(top.surface, 1, &[0, 0, 0]);
    h.f.wire.request(top.surface, 6, &[]);
    h.events.clear();
    h.settle();
    assert!(h.events.iter().any(|e| e.object == handle && e.opcode == 6));
    assert!(!h.events.iter().any(|e| e.object == child && e.opcode == 6));
    assert!(h.f.state.window_is_visible(&dialog));
}
