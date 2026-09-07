use super::{Duration, Instant, Policy, Schedule};

#[test]
fn late_window_coalesces_requests_and_never_renders_a_third_frame() {
    let now = Instant::now();
    let mut schedule = Schedule::with_policy(50_000, now, Policy::Deadline);
    assert!(schedule.render_due(now));
    assert_eq!(schedule.deadline(), None);
    schedule.rendered(Some(()), now);
    schedule.callbacks_sent(now);
    // No speculative work or refresh-rate polling when nobody requested work.
    assert_eq!(schedule.deadline(), Some(now + Duration::from_secs(3)));
    assert!(!schedule.render_due(now + Duration::from_secs(1)));

    let window = now + Duration::from_micros(17_500);
    for ms in [1, 5, 17] {
        schedule.request_redraw();
        assert!(!schedule.render_due(now + Duration::from_millis(ms)));
        assert_eq!(schedule.deadline(), Some(window));
    }
    assert!(!schedule.render_due(window - Duration::from_nanos(1)));
    assert!(schedule.render_due(window));
    // A late event does not retarget the window to another display period.
    let late = now + Duration::from_millis(25);
    assert!(schedule.render_due(late));
    schedule.rendered(Some(()), late);
    assert!(schedule.pending().is_some());
    assert!(schedule.queued().is_some());
    assert!(!schedule.callbacks_due(late));
    schedule.request_redraw();
    assert!(!schedule.render_due(late));
    assert_eq!(schedule.deadline(), Some(now + Duration::from_secs(3)));

    let submit = now + Duration::from_millis(26);
    assert_eq!(
        schedule.presented(now + Duration::from_millis(20), submit),
        Some(())
    );
    assert!(schedule.queued().is_none());
    let next_window = now + Duration::from_micros(37_500);
    assert_eq!(schedule.deadline(), Some(next_window));
    assert!(!schedule.render_due(submit));
    assert!(schedule.render_due(next_window));
    // The coalesced third request survives promotion, but still obeys the window.
    schedule.rendered(Some(()), next_window);
    assert_eq!(
        schedule.presented(
            now + Duration::from_millis(40),
            now + Duration::from_millis(41)
        ),
        Some(())
    );
    assert!(!schedule.render_due(now + Duration::from_millis(41)));
    let last = now + Duration::from_millis(60);
    assert_eq!(schedule.presented(last, last), Some(()));
    schedule.callbacks_sent(last);
    assert_eq!(schedule.deadline(), None);
    assert!(!schedule.render_due(last + Duration::from_secs(10)));
}

#[test]
fn empty_successor_consumes_request_without_idle_polling_or_watchdog_reset() {
    let now = Instant::now();
    let mut schedule = Schedule::with_policy(50_000, now, Policy::Deadline);
    schedule.rendered(Some(()), now);
    schedule.callbacks_sent(now);
    schedule.request_redraw();
    let window = now + Duration::from_micros(17_500);
    schedule.rendered(None, window);
    assert_eq!(schedule.queued(), None);
    assert!(!schedule.render_due(window));
    schedule.callbacks_sent(window);
    let flip = now + Duration::from_millis(20);
    assert_eq!(schedule.deadline(), Some(flip));
    assert!(schedule.stalled(now + Duration::from_secs(3)));
    assert_eq!(schedule.presented(flip, flip), Some(()));
    // An expired empty-frame wake cannot be downgraded by the same-batch flip.
    assert!(schedule.callbacks_advance_cycle(flip));
    schedule.callbacks_sent(flip);
    assert_eq!(schedule.deadline(), None);
    assert!(!schedule.callbacks_due(flip + Duration::from_secs(1)));
    assert!(!schedule.render_due(flip + Duration::from_secs(1)));
}
