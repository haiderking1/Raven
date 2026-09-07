use super::{Duration, Instant, Policy, Schedule};

#[test]
fn pressure_enables_overlap_then_drains_it_without_delaying_light_work() {
    let now = Instant::now();
    let mut schedule = Schedule::with_policy(50_000, now, Policy::Adaptive);
    schedule.rendered(Some(()), now);
    schedule.callbacks_sent(now);
    schedule.observe_render(Duration::from_micros(300));
    schedule.observe_gpu(Duration::from_micros(200));
    schedule.request_redraw();
    // Fast composition retains the checkpoint policy, not an extra queued frame.
    assert!(!schedule.render_due(now + Duration::from_millis(19)));
    assert_eq!(schedule.deadline(), Some(now + Duration::from_secs(3)));

    schedule.observe_render(Duration::from_millis(15));
    let deadline = schedule.deadline().unwrap();
    assert!(deadline > now && deadline < now + Duration::from_millis(20));
    assert!(schedule.render_due(deadline));
    schedule.rendered(Some(()), deadline);
    schedule.request_redraw();
    // Cooling down cannot discard a frame that Smithay still owns.
    for _ in 0..64 {
        schedule.observe_render(Duration::from_micros(300));
        schedule.observe_gpu(Duration::from_micros(200));
    }
    assert!(schedule.queued().is_some());
    let flip = now + Duration::from_millis(20);
    assert_eq!(schedule.presented(flip, flip), Some(()));
    assert!(schedule.pending().is_some());
    assert_eq!(schedule.deadline(), Some(flip + Duration::from_secs(3)));
    assert!(!schedule.render_due(now + Duration::from_millis(39)));
    let next = now + Duration::from_millis(40);
    assert_eq!(schedule.presented(next, next), Some(()));
    assert!(schedule.render_due(next));
}
