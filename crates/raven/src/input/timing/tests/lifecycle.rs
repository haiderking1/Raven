use super::{InputTiming, Observations, snapshots::Pending};
use std::time::{Duration, Instant};

#[test]
fn reporting_preserves_both_snapshots_and_reset_drops_them() {
    let now = Instant::now();
    let mut timing = InputTiming::new(now, 0);
    let mut observations = Observations::default();
    observations.add(Some(Duration::from_millis(1)), None);
    timing
        .snapshots
        .accept(observations, Some(Duration::from_millis(2)), false);
    observations.add(Some(Duration::from_millis(3)), None);
    timing
        .snapshots
        .accept(observations, Some(Duration::from_millis(4)), true);

    let report = timing.report(timing.deadline()).unwrap();
    assert!(report.pending && report.successor && !report.pending_ambiguous);
    assert!(timing.snapshots.has_pending() && timing.snapshots.has_successor());
    let completion = timing
        .snapshots
        .complete(Some(Duration::from_millis(5)), true);
    let Some(Pending::Snapshot(old)) = completion.presented else {
        panic!("report must retain the old frame");
    };
    assert_eq!(old.observations.count, 1);
    // Refill the software slot so reset must clear both occupied slots.
    timing
        .snapshots
        .accept(observations, Some(Duration::from_millis(6)), true);
    timing.reset(now);
    assert!(!timing.snapshots.has_pending() && !timing.snapshots.has_successor());
    assert!(
        timing
            .snapshots
            .complete(Some(Duration::from_millis(7)), false)
            .presented
            .is_none()
    );
}
