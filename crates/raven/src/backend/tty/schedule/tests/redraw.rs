use super::{Duration, Instant, Schedule};

#[test]
fn redraw_request_bypasses_empty_render_callback_wait() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    assert!(schedule.active());
    assert!(schedule.render_due(now));
    assert_eq!(schedule.deadline(), None);

    let empty = now + Duration::from_millis(2);
    schedule.rendered(None, empty);
    schedule.callbacks_sent(empty);
    let callbacks = now + Duration::from_nanos(16_666_666);
    assert_eq!(schedule.deadline(), Some(callbacks));
    assert!(!schedule.render_due(empty));
    assert!(!schedule.callbacks_due(now + Duration::from_millis(3)));

    schedule.request_redraw();
    let queued = now + Duration::from_millis(3);
    assert!(schedule.render_due(queued));
    schedule.rendered(Some(()), queued);
    assert!(!schedule.render_due(queued));
    assert!(schedule.callbacks_due(queued));
    assert!(schedule.callbacks_advance_cycle(queued));
    schedule.callbacks_sent(queued);
    assert!(!schedule.callbacks_due(callbacks));
    assert_eq!(schedule.deadline(), Some(queued + Duration::from_secs(3)));
}

#[test]
fn pending_coalesces_requests_and_presentation_does_not_request_redraw() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    schedule.rendered(Some(()), now);
    assert!(schedule.callbacks_due(now));
    assert!(schedule.callbacks_advance_cycle(now));
    schedule.callbacks_sent(now);
    schedule.request_redraw();
    schedule.request_redraw();
    assert!(!schedule.render_due(now));

    let flip = now + Duration::from_nanos(16_666_666);
    assert_eq!(schedule.presented(flip, flip), Some(()));
    assert!(schedule.render_due(flip));
    assert!(schedule.callbacks_due(flip));
    assert!(!schedule.callbacks_advance_cycle(flip));
    assert_eq!(schedule.deadline(), None);
    assert_eq!(
        schedule.presented(
            flip + Duration::from_millis(1),
            flip + Duration::from_millis(1)
        ),
        None
    );

    // Repaint before revisiting callbacks so new buffers are latched first.
    schedule.rendered(Some(()), flip);
    assert!(schedule.callbacks_advance_cycle(flip));
    schedule.callbacks_sent(flip);
    assert!(!schedule.render_due(flip));
    let next_flip = flip + Duration::from_nanos(16_666_666);
    assert_eq!(schedule.presented(next_flip, next_flip), Some(()));
    assert!(!schedule.render_due(next_flip));
    assert!(!schedule.callbacks_advance_cycle(next_flip));
    schedule.callbacks_sent(next_flip);
    assert_eq!(schedule.deadline(), None);
}
