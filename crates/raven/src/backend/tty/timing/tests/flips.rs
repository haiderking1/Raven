use super::super::flip::{Flips, elapsed};
use smithay::backend::drm::{DrmEventMetadata, DrmEventTime};
use std::time::{Duration, SystemTime};

fn mono(ms: u64, sequence: u32) -> Option<DrmEventMetadata> {
    Some(DrmEventMetadata {
        time: DrmEventTime::Monotonic(Duration::from_millis(ms)),
        sequence,
    })
}

fn realtime(ms: u64, sequence: u32) -> Option<DrmEventMetadata> {
    Some(DrmEventMetadata {
        time: DrmEventTime::Realtime(SystemTime::UNIX_EPOCH + Duration::from_millis(ms)),
        sequence,
    })
}

#[test]
fn flip_intervals_use_kernel_timestamps_and_wrapping_sequences() {
    let mut flips = Flips::default();
    assert!(flips.observe(mono(10, u32::MAX)).interval.is_none());
    let next = flips.observe(mono(16, 0));
    assert_eq!(next.interval, Some(Duration::from_millis(6)));
    assert_eq!(next.step, Some(1));
    // A late event dispatch must not be mistaken for a late scanout.
    assert_eq!(
        elapsed(mono(16, 0).unwrap().time, mono(200, 0).unwrap().time),
        Some(Duration::from_millis(184))
    );
    let next = flips.observe(mono(28, 2));
    assert_eq!(next.interval, Some(Duration::from_millis(12)));
    assert_eq!(next.step, Some(2));
    assert!(!next.clock_discontinuity);
    assert!(!next.sequence_repeated);
    assert!(!next.sequence_reset);
}

#[test]
fn missing_or_discontinuous_metadata_restarts_the_flip_baseline() {
    let mut flips = Flips::default();
    flips.observe(mono(10, 1));
    flips.observe(None);
    assert!(flips.observe(mono(100, 15)).interval.is_none());
    let switched = flips.observe(realtime(106, 16));
    assert!(switched.clock_discontinuity);
    assert_eq!(switched.step, Some(1));
    assert!(flips.observe(realtime(100, 17)).clock_discontinuity);
    assert_eq!(
        flips.observe(realtime(108, 18)).interval,
        Some(Duration::from_millis(8))
    );
    // A counter reset is not billions of missed refreshes; wrapping by one is tested above.
    let reset = flips.observe(realtime(114, 1));
    assert_eq!(reset.interval, Some(Duration::from_millis(6)));
    assert!(!reset.clock_discontinuity);
    assert!(reset.sequence_reset);
    assert_eq!(reset.step, None);
    assert_eq!(flips.observe(realtime(120, 2)).step, Some(1));
    assert_eq!(flips.clock_name(), "realtime");
}

#[test]
fn constant_sequences_preserve_valid_flip_timestamps() {
    let mut flips = Flips::default();
    flips.observe(mono(10, 0));
    let next = flips.observe(mono(16, 0));
    assert_eq!(next.interval, Some(Duration::from_millis(6)));
    assert_eq!(next.step, None);
    assert!(next.sequence_repeated);
    assert!(!next.clock_discontinuity);
    assert!(!next.sequence_reset);
}
