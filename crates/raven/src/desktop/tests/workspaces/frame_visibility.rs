use super::{
    assertions::frame_done,
    fixture::{fixture, frame, map},
};
use smithay::backend::renderer::element::{
    RenderElementPresentationState, RenderElementState, RenderElementStates,
};
use std::time::Duration;

#[test]
fn callbacks_follow_visible_pixels_and_occluded_clients_do_not_spin_or_starve() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (top, window) = map(&mut f, buffer);
    let surface = window.toplevel().unwrap().wl_surface().clone();
    let output = f.state.output.clone().unwrap();
    let mut rendered = RenderElementStates::default();
    rendered.states.insert(
        (&surface).into(),
        RenderElementState {
            visible_area: 100,
            presentation_state: RenderElementPresentationState::Rendering { reason: None },
        },
    );
    f.state.update_render_visibility(&output, &rendered);
    let first = frame(&mut f, top);
    f.state.send_frames(Duration::from_millis(1));
    assert!(frame_done(&f.dispatch(), first));
    let second = frame(&mut f, top);
    f.state.resend_frames(Duration::from_millis(2));
    assert!(!frame_done(&f.dispatch(), second));
    f.state.send_frames(Duration::from_millis(3));
    assert!(frame_done(&f.dispatch(), second));

    // A render-state entry alone is insufficient: zero visible pixels is occluded.
    rendered
        .states
        .get_mut(&(&surface).into())
        .unwrap()
        .visible_area = 0;
    f.state.update_render_visibility(&output, &rendered);
    let background = frame(&mut f, top);
    f.state.send_frames(Duration::from_millis(4));
    assert!(!frame_done(&f.dispatch(), background));
    let due = f.state.start_time + Duration::from_millis(253);
    assert_eq!(
        f.state
            .background_frame_deadline(Duration::from_millis(252)),
        Some(due)
    );
    assert!(!frame_done(&f.dispatch(), background));
    assert_eq!(
        f.state
            .background_frame_deadline(Duration::from_millis(253)),
        None
    );
    assert!(frame_done(&f.dispatch(), background));
    assert_eq!(
        f.state.background_frame_deadline(Duration::from_secs(10)),
        None
    );

    let hidden = frame(&mut f, top);
    f.state.switch_workspace(1);
    assert_eq!(
        f.state.background_frame_deadline(Duration::from_secs(11)),
        None
    );
    assert!(!frame_done(&f.dispatch(), hidden));
}
