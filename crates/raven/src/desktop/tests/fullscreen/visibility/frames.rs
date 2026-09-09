use super::super::{assertions::*, fixture::*};
use crate::desktop::tests::layers::assertions::frame_done;
use smithay::backend::renderer::element::{
    RenderElementPresentationState, RenderElementState, RenderElementStates,
};
use std::time::Duration;

#[test]
fn hidden_window_callbacks_ignore_stale_render_visibility_and_resume_on_exit() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (background, hidden) = map(&mut f, buffer);
    let (top, window) = map(&mut f, buffer);
    let output = f.state.output.clone().unwrap();
    let mut rendered = RenderElementStates::default();
    for window in [&hidden, &window] {
        rendered.states.insert(
            window.toplevel().unwrap().wl_surface().into(),
            RenderElementState {
                visible_area: 100,
                presentation_state: RenderElementPresentationState::Rendering { reason: None },
            },
        );
    }
    f.state.update_render_visibility(&output, &rendered);
    enter(&mut f, top);
    let hidden_callback = frame(&mut f, background);
    let visible_callback = frame(&mut f, top);
    f.state.send_frames(Duration::from_millis(1));
    let events = f.dispatch();
    assert!(frame_done(&events, visible_callback));
    assert!(!frame_done(&events, hidden_callback));
    let serial = configured(&request(&mut f, top, false), top, (400, 600), false);
    ack(&mut f, top, serial);
    commit(&mut f, top);
    f.state.update_render_visibility(&output, &rendered);
    f.state.send_frames(Duration::from_millis(2));
    assert!(frame_done(&f.dispatch(), hidden_callback));
}
