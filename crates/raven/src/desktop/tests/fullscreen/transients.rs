mod fixture;

use self::fixture::{dialog, not_tiled, parent};
use super::{assertions::*, fixture::*};
use crate::desktop::tests::layers::assertions::{focus, hit};
use smithay::reexports::wayland_server::Resource;

#[test]
fn visible_dialogs_keep_focus_and_stack_above_raised_owner_but_unrelated_dialogs_do_not() {
    let mut f = fixture();
    let large = f.buffer_sized(800, 600);
    let small = f.buffer_sized(200, 120);
    let (top, fullscreen) = startup(&mut f, large);
    let (child, transient) = dialog(&mut f, top, small);
    focus(&f, child.surface);
    let (other, hidden) = map(&mut f, large);
    let unrelated = f.toplevel();
    parent(&mut f, unrelated, Some(other));
    f.configure(unrelated);
    f.attach(unrelated, small);
    let unrelated_window = window(&f, unrelated);
    assert!(!f.state.window_is_visible(&hidden));
    assert!(!f.state.window_is_visible(&unrelated_window));
    focus(&f, child.surface);
    f.state.activate_window(Some(unrelated_window.clone()));
    focus(&f, child.surface);
    assert_eq!(
        f.state.visible_windows().collect::<Vec<_>>(),
        vec![&fullscreen, &transient]
    );
    let members = [&fullscreen, &hidden];
    for floating in [&transient, &unrelated_window] {
        assert!(f.state.window_is_floating(floating));
        assert!(f.state.window_tile_geometry(floating).is_none());
    }
    let tiles: Vec<_> = members
        .iter()
        .map(|w| f.state.window_tile_geometry(w).unwrap())
        .collect();

    f.state.activate_window(Some(fullscreen.clone()));
    assert_eq!(
        f.state.visible_windows().collect::<Vec<_>>(),
        vec![&fullscreen, &transient]
    );
    hit(&f, (400.0, 300.0), child.surface);
    f.state.focus_window_at((400.0, 300.0).into());
    focus(&f, child.surface);
    parent(&mut f, child, Some(other));
    assert!(!f.state.window_is_visible(&transient));
    focus(&f, top.surface);
    hit(&f, (400.0, 300.0), top.surface);
    parent(&mut f, child, Some(top));
    assert!(f.state.window_is_visible(&transient));
    focus(&f, top.surface); // Reparenting alone must not activate the dialog.
    parent(&mut f, child, None);
    assert!(!f.state.window_is_visible(&transient));
    parent(&mut f, child, Some(top));

    // Focus and tiling configures during exit must retain the in-flight mode/size.
    let serial = configured(&request(&mut f, top, false), top, (400, 600), false);
    f.state.activate_window(Some(transient.clone()));
    f.state.retile_workspace(0);
    ack(&mut f, top, serial);
    commit(&mut f, top);
    owner(&f, None);
    for (member, tile) in members.into_iter().zip(tiles) {
        assert_eq!(f.state.window_layout_geometry(member), Some(tile));
        assert_eq!(f.state.space().element_location(member), Some(tile.loc));
        assert!(f.state.window_is_visible(member));
    }
    layout(&f, &transient, (100, 240), (200, 120));
    layout(&f, &unrelated_window, (500, 240), (200, 120));
    assert!(f.state.window_is_visible(&transient));
    assert!(f.state.window_is_visible(&unrelated_window));
}

#[test]
fn nested_dialog_geometry_is_centered_bounded_and_loses_visibility_with_its_parent() {
    let mut f = fixture();
    let output = f.state.output.clone().unwrap();
    f.state.space_mut().map_output(&output, (40, 30));
    let small = f.buffer();
    let (top, fullscreen) = startup(&mut f, small);
    let buffer = f.buffer_sized(220, 160);
    let (child, transient) = dialog(&mut f, top, buffer);
    f.wire.request(child.xdg, 3, &[10, 20, 180, 100]);
    let events = commit(&mut f, child);
    configured(&events, child, (180, 100), false);
    not_tiled(&events, child);
    layout(&f, &transient, (350, 280), (180, 100));
    let (surface, origin) = f.state.surface_under((351.0, 281.0).into()).unwrap();
    assert_eq!(surface.id().protocol_id(), child.surface);
    assert_eq!(origin, (340.0, 260.0).into());
    assert!(f.state.surface_under((45.0, 35.0).into()).is_none());

    let (nested, descendant) = dialog(&mut f, child, small);
    assert_eq!(f.state.fullscreen_transient_depth(&descendant), Some(2));
    layout(&f, &descendant, (390, 280), (100, 100));
    f.state.activate_window(Some(fullscreen.clone()));
    f.state.activate_window(Some(transient.clone()));
    assert_eq!(
        f.state.visible_windows().collect::<Vec<_>>(),
        vec![&fullscreen, &transient, &descendant]
    );
    hit(&f, (400.0, 300.0), nested.surface);
    f.state.focus_window_at((400.0, 300.0).into());
    focus(&f, nested.surface);
    f.dispatch(); // Drain the activation configures before checking resize events.

    let oversized = f.buffer_sized(1000, 800);
    f.wire.request(nested.xdg, 3, &[12, 16, 950, 750]);
    let events = crate::desktop::tests::workspaces::fixture::attach(&mut f, nested, oversized);
    configured(&events, nested, (800, 600), false);
    not_tiled(&events, nested);
    layout(&f, &descendant, (40, 30), (800, 600));
    assert_eq!(
        f.state.window_surface_origin(&descendant),
        Some((28, 14).into())
    );
    hit(&f, (839.0, 629.0), nested.surface);
    assert!(f.state.surface_under((840.0, 630.0).into()).is_none());

    f.attach(child, 0);
    assert!(!f.state.window_is_visible(&transient));
    assert!(!f.state.window_is_visible(&descendant));
    focus(&f, top.surface);
    hit(&f, (400.0, 300.0), top.surface);
    // Remapping the ancestor makes the still-mapped nested child visible again.
    parent(&mut f, child, Some(top));
    f.configure(child);
    f.attach(child, buffer);
    assert!(f.state.window_is_visible(&descendant));
    f.state.activate_window(Some(descendant.clone()));
    destroy(&mut f, child);
    assert!(!f.state.window_is_visible(&descendant));
    focus(&f, top.surface);
}

#[test]
fn new_dialog_inherits_hidden_parent_workspace_without_following_or_focus_theft() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (top, fullscreen) = startup(&mut f, buffer);
    f.state.switch_workspace(1);
    let (active, _) = map(&mut f, buffer);
    let child = f.toplevel();
    no_configure(&parent(&mut f, child, Some(top)), child);
    let transient = window(&f, child);
    assert_eq!(f.state.workspaces.index_of(&transient), Some(0));
    let serial = configured(&commit(&mut f, child), child, (0, 0), false);
    ack(&mut f, child, serial);
    f.attach(child, buffer);
    assert_eq!(f.state.workspaces.active, 1);
    assert!(!f.state.window_is_visible(&transient));
    focus(&f, active.surface);

    let nested = f.toplevel();
    no_configure(&parent(&mut f, nested, Some(child)), nested);
    let descendant = window(&f, nested);
    assert_eq!(f.state.workspaces.index_of(&descendant), Some(0));
    f.configure(nested);
    f.attach(nested, buffer);
    assert_eq!(f.state.workspaces.active, 1);
    focus(&f, active.surface);
    f.state.switch_workspace(0);
    owner(&f, Some(&fullscreen));
    assert_eq!(
        f.state.visible_windows().collect::<Vec<_>>(),
        vec![&fullscreen, &transient, &descendant]
    );
    hit(&f, (400.0, 300.0), nested.surface);
    f.state.focus_window_at((400.0, 300.0).into());
    focus(&f, nested.surface);
}
