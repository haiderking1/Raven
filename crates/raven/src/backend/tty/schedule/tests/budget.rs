use super::{Duration, Instant, Policy, Schedule};

#[test]
fn independent_cpu_gpu_samples_sum_with_margin_then_decay_and_cap_at_one_period() {
    let now = Instant::now();
    let mut schedule = Schedule::with_policy(50_000, now, Policy::Deadline);
    schedule.rendered(Some(()), now + Duration::from_millis(5));
    schedule.request_redraw();
    schedule.observe_render(Duration::from_millis(4));
    schedule.observe_gpu(Duration::from_millis(6));
    // Sum, not max: CPU and GPU can overlap but are not assumed to overlap.
    assert_eq!(
        schedule.deadline(),
        Some(now + Duration::from_micros(9_500))
    );
    schedule.observe_render(Duration::ZERO);
    schedule.observe_gpu(Duration::ZERO);
    // Both estimates release 1/16 of the distance to the new sample.
    assert_eq!(
        schedule.deadline(),
        Some(now + Duration::from_micros(10_125))
    );
    schedule.observe_render(Duration::MAX);
    schedule.observe_gpu(Duration::MAX);
    assert_eq!(schedule.deadline(), Some(now));
    assert!(schedule.render_due(now + Duration::from_millis(5)));

    let resumed = now + Duration::from_secs(1);
    schedule.resume(resumed);
    schedule.rendered(Some(()), resumed);
    schedule.request_redraw();
    assert_eq!(
        schedule.deadline(),
        Some(resumed + Duration::from_micros(17_500))
    );
}
