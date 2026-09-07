use super::super::observations::Ages;
use super::{Observations, Pending, Queue, Snapshot};
use std::time::Duration;

fn time(ms: u64) -> Option<Duration> {
    Some(Duration::from_millis(ms))
}

fn observations(ms: u64) -> Observations {
    let mut observations = Observations::default();
    observations.add(time(ms), time(ms + 1));
    observations
}

fn snapshot(pending: Option<Pending>) -> Snapshot {
    match pending {
        Some(Pending::Snapshot(snapshot)) => snapshot,
        _ => panic!("expected an attributable snapshot"),
    }
}

#[test]
fn fifo_keeps_observations_and_acceptance_time_but_stamps_promotion() {
    let mut queue = Queue::default();
    assert!(
        !queue
            .accept(observations(5), time(10), false)
            .invariant_mismatch
    );
    // The successor may be accepted after the old frame's kernel timestamp.
    assert!(
        !queue
            .accept(observations(22), time(25), true)
            .invariant_mismatch
    );
    assert!(queue.has_pending() && queue.has_successor());
    assert!(!queue.is_ambiguous());
    assert!(queue.successor.as_ref().unwrap().submitted.is_none());

    let first = queue.complete(time(30), true);
    assert!(!first.invariant_mismatch && !first.successor_discarded);
    assert_eq!(first.promotion, Some((time(25), time(30))));
    let first = snapshot(first.presented);
    assert_eq!((first.queued, first.submitted), (time(10), time(10)));
    let mut ages = Ages::default();
    ages.record(first.observations, time(20));
    assert_eq!(ages.source_oldest.to_string(), "1:15.000/15.000/15.000");
    assert_eq!(ages.dispatch_latest.to_string(), "1:14.000/14.000/14.000");
    assert!(queue.has_pending() && !queue.has_successor());

    let second = snapshot(queue.complete(time(60), false).presented);
    assert_eq!((second.queued, second.submitted), (time(25), time(30)));
    let mut ages = Ages::default();
    ages.record(second.observations, time(50));
    assert_eq!(ages.source_latest.to_string(), "1:28.000/28.000/28.000");
    assert_eq!(ages.dispatch_oldest.to_string(), "1:27.000/27.000/27.000");
    assert!(!queue.has_pending() && !queue.has_successor());
}

#[test]
fn failed_successor_is_discarded_without_losing_old_presentation() {
    let mut queue = Queue::default();
    queue.accept(observations(5), time(10), false);
    queue.accept(observations(15), time(20), true);
    let completion = queue.complete(time(30), false);
    assert!(completion.successor_discarded);
    assert!(!completion.invariant_mismatch);
    assert!(completion.promotion.is_none());
    assert_eq!(snapshot(completion.presented).queued, time(10));
    assert!(!queue.has_pending() && !queue.has_successor());
    assert!(queue.complete(time(40), false).presented.is_none());
    assert!(
        !queue
            .accept(observations(45), time(50), false)
            .invariant_mismatch
    );
    assert_eq!(
        snapshot(queue.complete(time(60), false).presented).queued,
        time(50)
    );
}

#[test]
fn invalid_acceptance_counts_every_discard_and_never_overwrites() {
    // Orphan successor, overlapping direct submission, and third snapshot.
    for (accepted, deferred, discarded) in [(0, true, 1), (1, false, 2), (2, true, 3)] {
        let mut queue = Queue::default();
        if accepted > 0 {
            queue.accept(observations(5), time(10), false);
        }
        if accepted > 1 {
            queue.accept(observations(15), time(20), true);
        }
        let result = queue.accept(observations(25), time(30), deferred);
        assert!(result.invariant_mismatch);
        assert_eq!(result.discarded, discarded);
        assert!(queue.is_ambiguous() && !queue.has_successor());
        // An unknown submitted successor cannot restore attribution.
        assert!(matches!(
            queue.complete(time(40), true).presented,
            Some(Pending::Ambiguous)
        ));
        assert!(queue.is_ambiguous());
        assert!(matches!(
            queue.complete(time(50), false).presented,
            Some(Pending::Ambiguous)
        ));
        assert!(!queue.has_pending());
    }
}

#[test]
fn submission_without_successor_preserves_old_snapshot_and_marks_unknown_next() {
    let mut queue = Queue::default();
    queue.accept(observations(5), time(10), false);
    let completion = queue.complete(time(30), true);
    assert_eq!(snapshot(completion.presented).queued, time(10));
    assert!(completion.invariant_mismatch);
    assert!(completion.promotion.is_none());
    assert!(queue.is_ambiguous() && !queue.has_successor());
}

#[test]
fn unavailable_submission_clock_does_not_reuse_acceptance_clock() {
    let mut queue = Queue::default();
    queue.accept(observations(5), time(10), false);
    queue.accept(observations(15), time(20), true);
    assert_eq!(queue.complete(None, true).promotion, Some((time(20), None)));
    let successor = snapshot(queue.complete(time(50), false).presented);
    assert_eq!(successor.queued, time(20));
    assert_eq!(successor.submitted, None);
}
