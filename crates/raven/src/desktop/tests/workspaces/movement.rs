use super::{assertions::*, fixture::*};

#[test]
fn transfer_reflows_both_workspaces_and_hidden_lifecycle_keeps_ownership() {
    let mut f = fixture();
    f.state.pointer_location = (10.0, 10.0).into();
    let buffer = f.buffer();
    let (moving, moved) = map(&mut f, buffer);
    let (remaining, source) = map(&mut f, buffer);
    f.state.switch_workspace(9);
    let (target_first, target_master) = map(&mut f, buffer);
    let (target_second, target_stack) = map(&mut f, buffer);

    // Ownership starts at get_toplevel, before even the initial surface commit.
    let delayed = f.toplevel();
    f.dispatch();
    let delayed_window = window(&f, delayed);
    assert_eq!(f.state.workspaces.index_of(&delayed_window), Some(9));
    f.state.switch_workspace(0);
    f.state.activate_window(Some(moved.clone()));
    f.dispatch();
    f.state.move_focused_to_workspace(9);
    let events = f.dispatch();
    size(&events, remaining, (800, 600));
    size(&events, target_second, (400, 300));
    size(&events, moving, (400, 300));
    focus(&f, 0, Some(&source));
    pointer(&f, Some(remaining));
    layout(&f, 0, &[(&source, (0, 0))]);
    let target_tiles = [
        (&target_master, (0, 0)),
        (&target_stack, (400, 0)),
        (&moved, (400, 300)),
    ];
    layout(&f, 9, &target_tiles);
    assert_eq!(
        f.state.workspaces.entries[9].focused.as_ref(),
        Some(&target_stack)
    );
    output_membership(&mut f);

    // A hidden buffer commit must still update the window, not map it here.
    f.wire.request(moving.xdg, 3, &[0, 0, 80, 70]);
    let replacement = f.buffer();
    f.attach(moving, replacement);
    assert_eq!(moved.geometry().size, (80, 70).into());
    layout(&f, 9, &target_tiles);
    focus(&f, 0, Some(&source));

    let events = attach(&mut f, moving, 0);
    size(&events, target_second, (400, 600));
    layout(
        &f,
        9,
        &[(&target_master, (0, 0)), (&target_stack, (400, 0))],
    );
    assert_eq!(f.state.workspaces.index_of(&moved), Some(9));
    focus(&f, 0, Some(&source));
    f.configure(moving);
    let events = attach(&mut f, moving, replacement);
    size(&events, target_second, (400, 300));
    layout(&f, 9, &target_tiles);
    focus(&f, 0, Some(&source));
    pointer(&f, Some(remaining));
    output_membership(&mut f);

    f.wire.request(moving.role, 0, &[]);
    f.wire.request(moving.xdg, 0, &[]);
    f.wire.request(moving.surface, 0, &[]);
    let events = f.dispatch();
    size(&events, target_second, (400, 600));
    assert_eq!(f.state.workspaces.index_of(&moved), None);
    assert!(!f.state.windows.contains(&moved));
    layout(
        &f,
        9,
        &[(&target_master, (0, 0)), (&target_stack, (400, 0))],
    );
    focus(&f, 0, Some(&source));

    f.configure(delayed);
    let events = attach(&mut f, delayed, buffer);
    size(&events, target_second, (400, 300));
    let final_tiles = [
        (&target_master, (0, 0)),
        (&target_stack, (400, 0)),
        (&delayed_window, (400, 300)),
    ];
    layout(&f, 9, &final_tiles);
    layout(&f, 0, &[(&source, (0, 0))]);
    focus(&f, 0, Some(&source));
    pointer(&f, Some(remaining));
    output_membership(&mut f);
    f.state.switch_workspace(9);
    layout(&f, 9, &final_tiles);
    focus(&f, 9, Some(&target_stack));
    pointer(&f, Some(target_first));
}
