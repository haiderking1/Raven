use super::super::fixture::Fixture;
use crate::backend::tty::TestQueuedFeedback as QueuedFeedback;
use smithay::{
    backend::renderer::element::{
        RenderElementPresentationState, RenderElementState, RenderElementStates,
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_protocols::wp::presentation_time::server::wp_presentation_feedback::Kind,
    utils::Monotonic,
    wayland::presentation::Refresh,
};
use std::time::Duration;

#[test]
fn a_failed_successor_cannot_discard_or_steal_the_completed_frames_feedback() {
    let mut f = Fixture::new();
    let output = Output::new(
        "queued-feedback-test".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (800, 600).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    f.state.space_mut().map_output(&output, (0, 0));
    f.state.output = Some(output.clone());
    let presentation = super::super::globals::bind(&mut f, "wp_presentation", 1);
    let top = f.toplevel();
    f.configure(top);
    let a = f.id();
    f.wire.request(presentation, 1, &[top.surface, a]);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let surface = f
        .state
        .space()
        .elements()
        .next()
        .unwrap()
        .toplevel()
        .unwrap()
        .wl_surface()
        .clone();
    let mut rendered = RenderElementStates::default();
    rendered.states.insert(
        (&surface).into(),
        RenderElementState {
            visible_area: 100,
            presentation_state: RenderElementPresentationState::Rendering { reason: None },
        },
    );
    let drm_a = QueuedFeedback::default();
    drm_a.set(f.state.take_presentation_feedback(&output, &rendered, None));
    let scheduler_a = drm_a.clone();

    let b = f.id();
    f.wire.request(presentation, 1, &[top.surface, b]);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let drm_b = QueuedFeedback::default();
    drm_b.set(f.state.take_presentation_feedback(&output, &rendered, None));
    let scheduler_b = drm_b.clone();

    // Model the documented frame_submitted failure: it drops both userdata
    // values even though A completed and only the successor B was rejected.
    drop((drm_a, drm_b));
    let events = f.dispatch();
    assert!(
        !events
            .iter()
            .any(|event| (event.object == a || event.object == b)
                && (event.opcode == 1 || event.opcode == 2))
    );
    scheduler_a.take().unwrap().presented::<_, Monotonic>(
        Duration::from_secs(1),
        Refresh::Fixed(Duration::from_millis(16)),
        7,
        Kind::Vsync,
    );
    drop(scheduler_b);
    let events = f.dispatch();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.object == a && event.opcode == 1)
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.object == b && event.opcode == 2)
            .count(),
        1
    );
    assert!(
        !events
            .iter()
            .any(|event| (event.object == a && event.opcode == 2)
                || (event.object == b && event.opcode == 1))
    );
}
