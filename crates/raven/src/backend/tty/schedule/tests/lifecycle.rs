use super::{Duration, Instant, Schedule};

#[test]
fn pause_and_resume_clear_pending_callbacks_and_reset_callback_anchor() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    schedule.rendered(true, now);
    schedule.request_redraw();
    schedule.pause();
    assert!(!schedule.active());
    assert!(!schedule.render_due());
    assert_eq!(schedule.deadline(), None);
    assert!(!schedule.presented(now + Duration::from_secs(1)));
    assert!(!schedule.callbacks_due(now + Duration::from_secs(10)));
    assert!(!schedule.stalled(now + Duration::from_secs(10)));
    schedule.request_redraw();
    assert!(!schedule.render_due());

    let resumed = now + Duration::from_secs(10);
    schedule.resume(resumed);
    assert!(schedule.active());
    assert!(schedule.render_due());
    assert_eq!(schedule.deadline(), None);
    assert!(!schedule.presented(resumed));
    schedule.rendered(false, resumed + Duration::from_millis(2));
    assert_eq!(
        schedule.deadline(),
        Some(resumed + Duration::from_nanos(16_666_666))
    );
    assert!(schedule.callbacks_due(resumed + Duration::from_millis(2)));
    assert!(!schedule.callbacks_advance_cycle(resumed + Duration::from_millis(2)));
    schedule.callbacks_sent(resumed + Duration::from_millis(2));

    schedule.pause();
    assert_eq!(schedule.deadline(), None);
    assert!(!schedule.callbacks_due(resumed + Duration::from_nanos(16_666_666)));
    schedule.resume(resumed + Duration::from_secs(1));
    assert!(schedule.render_due());
    assert_eq!(schedule.deadline(), None);
}

#[test]
fn watchdog_only_tracks_the_current_pending_frame_and_resume_resets_it() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    assert!(!schedule.stalled(now + Duration::from_secs(10)));
    schedule.rendered(true, now);
    let expires = now + Duration::from_secs(3);
    assert_eq!(schedule.deadline(), Some(expires));
    assert!(!schedule.stalled(expires - Duration::from_nanos(1)));
    assert!(schedule.stalled(expires));
    schedule.request_redraw();
    assert_eq!(schedule.deadline(), Some(expires));

    schedule.resume(expires);
    assert!(!schedule.stalled(expires));
    assert!(schedule.render_due());
    assert_eq!(schedule.deadline(), None);
    schedule.rendered(true, expires);
    assert_eq!(schedule.deadline(), Some(expires + Duration::from_secs(3)));
    assert!(!schedule.stalled(expires));
    assert!(schedule.presented(expires + Duration::from_nanos(16_666_666)));
    assert!(!schedule.stalled(expires + Duration::from_secs(10)));
    schedule.callbacks_sent(expires + Duration::from_nanos(16_666_666));
    assert_eq!(schedule.deadline(), None);
}
