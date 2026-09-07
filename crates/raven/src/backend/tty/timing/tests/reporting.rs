use super::super::Timing;
use smithay::backend::drm::{DrmEventMetadata, DrmEventTime};
use std::time::{Duration, Instant};

fn flip(ms: u64, sequence: u32) -> Option<DrmEventMetadata> {
    Some(DrmEventMetadata {
        time: DrmEventTime::Monotonic(Duration::from_millis(ms)),
        sequence,
    })
}

#[test]
fn reports_separate_work_and_keep_flip_history_until_session_reset() {
    let now = Instant::now();
    let mut timing = Timing::new(165_000, now);
    timing.rendered(Duration::from_millis(1), true);
    timing.rendered(Duration::from_millis(3), true);
    timing.rendered(Duration::from_micros(100), false);
    let deadline = now + Duration::from_millis(6);
    timing.timer_wakeup(Some(deadline), now);
    timing.timer_wakeup(Some(deadline), now + Duration::from_millis(12));
    timing.timer_wakeup(None, now + Duration::from_millis(50));
    timing.presented(flip(10, u32::MAX));
    timing.presented(flip(16, 0));
    assert!(timing.report(now + Duration::from_millis(1999)).is_none());
    let report = timing.report(now + Duration::from_secs(2)).unwrap();
    assert_eq!(report.span, Duration::from_secs(2));
    assert_eq!(report.refresh_millihz, 165_000);
    let window = report.window;
    assert_eq!(window.draws.count, 2);
    assert_eq!(window.draws.min, Duration::from_millis(1));
    assert_eq!(window.draws.mean_ms(), Some(2.0));
    assert_eq!(window.draws.max, Duration::from_millis(3));
    assert_eq!(window.empty.count, 1);
    assert_eq!(window.empty.total, Duration::from_micros(100));
    assert_eq!(window.flips, 2);
    assert_eq!(window.flip_intervals.total, Duration::from_millis(6));
    assert_eq!(window.steps, [1, 0, 0]);
    assert_eq!(window.timer_early, 1);
    assert_eq!(window.timer_lateness.count, 1);
    assert_eq!(window.timer_lateness.total, Duration::from_millis(6));

    timing.presented(flip(28, 2));
    timing.presented(flip(34, 2));
    let report = timing.report(now + Duration::from_secs(4)).unwrap();
    assert_eq!(report.window.draws.count, 0);
    assert_eq!(report.window.draws.mean_ms(), None);
    assert_eq!(report.window.flips, 2);
    assert_eq!(
        report.window.flip_intervals.total,
        Duration::from_millis(18)
    );
    assert_eq!(report.window.flip_intervals.count, 2);
    assert_eq!(report.window.repeated_sequences, 1);
    assert_eq!(report.window.reset_sequences, 0);
    assert_eq!(report.window.clock_discontinuities, 0);
    assert_eq!(report.window.last_sequence, Some(2));
    assert_eq!(report.window.steps, [0, 1, 0]);
    assert_eq!(report.window.timer_early, 0);

    timing.reset(now + Duration::from_secs(10));
    timing.presented(flip(10_000, 100));
    assert!(timing.report(now + Duration::from_secs(11)).is_none());
    let report = timing.report(now + Duration::from_secs(12)).unwrap();
    assert_eq!(report.window.flips, 1);
    assert_eq!(report.window.flip_intervals.count, 0);
    assert_eq!(report.window.steps, [0, 0, 0]);
    assert_eq!(report.window.repeated_sequences, 0);
    assert_eq!(report.window.last_sequence, Some(100));
}
