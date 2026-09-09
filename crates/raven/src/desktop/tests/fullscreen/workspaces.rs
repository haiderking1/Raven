use super::super::layers::assertions::focus;
use super::{assertions::*, fixture::*};

#[test]
fn hidden_startup_and_commits_do_not_follow_or_steal_focus() {
    let mut f = fixture();
    let buffer = f.buffer();
    let delayed = f.toplevel();
    no_configure(&request(&mut f, delayed, true), delayed);
    let serial = configured(&commit(&mut f, delayed), delayed, (800, 600), true);
    ack(&mut f, delayed, serial);
    f.state.switch_workspace(1);
    let (active, visible) = map(&mut f, buffer);
    f.attach(delayed, buffer);
    let hidden = window(&f, delayed);
    assert_eq!(f.state.workspaces.active, 1);
    assert_eq!(f.state.workspaces.index_of(&hidden), Some(0));
    owner(&f, None);
    focus(&f, active.surface);
    assert!(!f.state.window_is_visible(&hidden));
    assert!(f.state.window_is_visible(&visible));
    f.state.switch_workspace(0);
    owner(&f, Some(&hidden));
    f.state.switch_workspace(1);
    // Even an already-fullscreen hidden client cannot bring its workspace forward.
    request(&mut f, delayed, true);
    commit(&mut f, delayed);
    assert_eq!(f.state.workspaces.active, 1);
    focus(&f, active.surface);
    f.attach(delayed, 0);
    f.configure(delayed);
    f.attach(delayed, buffer);
    assert_eq!(f.state.workspaces.active, 1);
    focus(&f, active.surface);
    f.state.switch_workspace(0);
    owner(&f, None);
    assert!(f.state.window_is_visible(&hidden));
}

#[test]
fn transfer_preserves_fullscreen_replaces_destination_owner_without_following() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (remaining, source) = map(&mut f, buffer);
    let (moving, moved) = startup(&mut f, buffer);
    f.state.switch_workspace(1);
    let (destination, incumbent) = startup(&mut f, buffer);
    f.state.switch_workspace(0);
    owner(&f, Some(&moved));
    f.state.move_focused_to_workspace(1);
    f.dispatch();
    assert_eq!(f.state.workspaces.active, 0);
    assert_eq!(f.state.workspaces.index_of(&moved), Some(1));
    owner(&f, None);
    focus(&f, remaining.surface);
    layout(&f, &source, (0, 0), (800, 600));
    assert!(!f.state.window_is_visible(&moved));
    f.state.switch_workspace(1);
    owner(&f, Some(&moved));
    layout(&f, &moved, (0, 0), (800, 600));
    assert!(!f.state.window_is_visible(&incumbent));
    focus(&f, moving.surface);
    f.state.switch_workspace(0);
    destroy(&mut f, moving);
    assert_eq!(f.state.workspaces.active, 0);
    focus(&f, remaining.surface);
    f.state.switch_workspace(1);
    owner(&f, None);
    assert!(f.state.window_is_visible(&incumbent));
    layout(&f, &incumbent, (0, 0), (800, 600));
    focus(&f, destination.surface);
}
