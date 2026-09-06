use super::*;

#[test]
fn pause_discards_pending_flip_and_resume_allows_a_fresh_frame() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    schedule.rendered(true, now);
    assert!(!schedule.due(now + schedule.interval()));
    assert!(schedule.stalled(now + FLIP_TIMEOUT));
    schedule.pause();
    assert!(!schedule.presented(now));
    assert!(!schedule.due(now + FLIP_TIMEOUT));
    assert!(!schedule.stalled(now + FLIP_TIMEOUT));
    schedule.resume(now + FLIP_TIMEOUT);
    assert!(schedule.due(now + FLIP_TIMEOUT));
    assert!(!schedule.stalled(now + FLIP_TIMEOUT));
}

#[test]
fn unchanged_frames_wait_for_timer_but_pageflips_allow_immediate_render() {
    let now = Instant::now();
    let mut schedule = Schedule::new(60_000, now);
    schedule.rendered(false, now);
    assert!(!schedule.due(now));
    let next = now + schedule.interval();
    assert!(schedule.due(next));
    schedule.rendered(true, next);
    assert!(schedule.presented(next));
    assert!(schedule.due(next));
    assert!(!schedule.presented(next));
}
