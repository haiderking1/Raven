use super::{Duration, Instant, Schedule};

#[test]
fn redraw_request_bypasses_empty_render_callback_wait() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    assert!(schedule.active());
    assert!(schedule.render_due());
    assert_eq!(schedule.deadline(), None);

    let empty = now + Duration::from_millis(2);
    schedule.rendered(false, empty);
    schedule.callbacks_sent(empty);
    let callbacks = now + Duration::from_nanos(16_666_666);
    assert_eq!(schedule.deadline(), Some(callbacks));
    assert!(!schedule.render_due());
    assert!(!schedule.callbacks_due(now + Duration::from_millis(3)));

    schedule.request_redraw();
    assert!(schedule.render_due());
    let queued = now + Duration::from_millis(3);
    schedule.rendered(true, queued);
    assert!(!schedule.render_due());
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
    schedule.rendered(true, now);
    assert!(schedule.callbacks_due(now));
    assert!(schedule.callbacks_advance_cycle(now));
    schedule.callbacks_sent(now);
    schedule.request_redraw();
    schedule.request_redraw();
    assert!(!schedule.render_due());

    let flip = now + Duration::from_nanos(16_666_666);
    assert!(schedule.presented(flip));
    assert!(schedule.render_due());
    assert!(schedule.callbacks_due(flip));
    assert!(!schedule.callbacks_advance_cycle(flip));
    assert_eq!(schedule.deadline(), None);
    assert!(!schedule.presented(flip + Duration::from_millis(1)));

    // Repaint before revisiting callbacks so new buffers are latched first.
    schedule.rendered(true, flip);
    assert!(schedule.callbacks_advance_cycle(flip));
    schedule.callbacks_sent(flip);
    assert!(!schedule.render_due());
    let next_flip = flip + Duration::from_nanos(16_666_666);
    assert!(schedule.presented(next_flip));
    assert!(!schedule.render_due());
    assert!(!schedule.callbacks_advance_cycle(next_flip));
    schedule.callbacks_sent(next_flip);
    assert_eq!(schedule.deadline(), None);
}
