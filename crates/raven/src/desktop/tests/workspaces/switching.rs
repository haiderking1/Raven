use super::{assertions::*, fixture::*};
use std::time::Duration;

#[test]
fn switching_restores_independent_tiles_and_focus_and_hides_frames() {
    let mut f = fixture();
    f.state.pointer_location = (410.0, 10.0).into();
    let buffer = f.buffer();
    let (first, master) = map(&mut f, buffer);
    let (second, stack) = map(&mut f, buffer);
    // Saved keyboard focus and pointer focus deliberately name different tiles.
    f.state.activate_window(Some(master.clone()));
    focus(&f, 0, Some(&master));
    pointer(&f, Some(second));
    layout(&f, 0, &[(&master, (0, 0)), (&stack, (400, 0))]);
    output_membership(&mut f);
    let hidden_callback = frame(&mut f, first);

    f.state.switch_workspace(9);
    focus(&f, 9, None);
    pointer(&f, None);
    assert_eq!(f.state.space().elements().count(), 0);
    output_membership(&mut f);

    let (_, other_master) = map(&mut f, buffer);
    let (other_second, other_stack) = map(&mut f, buffer);
    let (_, other_last) = map(&mut f, buffer);
    f.state.activate_window(Some(other_stack.clone()));
    let other_tiles = [
        (&other_master, (0, 0)),
        (&other_stack, (400, 0)),
        (&other_last, (400, 300)),
    ];
    layout(&f, 9, &other_tiles);
    focus(&f, 9, Some(&other_stack));
    pointer(&f, Some(other_second));
    output_membership(&mut f);
    let visible_callback = frame(&mut f, other_second);
    f.state.send_frames(Duration::from_millis(10));
    let events = f.dispatch();
    assert!(!frame_done(&events, hidden_callback));
    assert!(frame_done(&events, visible_callback));

    // Switching must not turn the raised window into the master tile.
    f.state.switch_workspace(0);
    focus(&f, 0, Some(&master));
    pointer(&f, Some(second));
    layout(&f, 0, &[(&master, (0, 0)), (&stack, (400, 0))]);
    layout(&f, 9, &other_tiles);
    output_membership(&mut f);
    f.state.send_frames(Duration::from_millis(20));
    assert!(frame_done(&f.dispatch(), hidden_callback));

    f.state.switch_workspace(9);
    focus(&f, 9, Some(&other_stack));
    pointer(&f, Some(other_second));
    layout(&f, 9, &other_tiles);
    output_membership(&mut f);
}
