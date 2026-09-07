use smithay::{
    backend::drm::{DrmEventMetadata, DrmEventTime},
    utils::{Clock, Monotonic},
};
use std::time::{Duration, SystemTime};

#[derive(Default)]
pub(super) struct Flips {
    previous: Option<DrmEventMetadata>,
}

#[derive(Default)]
pub(super) struct Observation {
    pub interval: Option<Duration>,
    pub step: Option<u32>,
    pub clock_discontinuity: bool,
    pub sequence_repeated: bool,
    pub sequence_reset: bool,
}

impl Flips {
    pub fn observe(&mut self, metadata: Option<DrmEventMetadata>) -> Observation {
        let previous = self.previous;
        // Missing metadata breaks the chain; never bridge it with dispatch times.
        self.previous = metadata;
        let (Some(previous), Some(current)) = (previous, metadata) else {
            return Observation::default();
        };
        let interval = elapsed(previous.time, current.time).filter(|value| !value.is_zero());
        let step = current.sequence.wrapping_sub(previous.sequence);
        // Timestamp and sequence validity are independent. A driver may report
        // a constant sequence while still supplying fresh pageflip timestamps.
        Observation {
            interval,
            step: (step != 0 && step <= u32::MAX / 2).then_some(step),
            clock_discontinuity: interval.is_none(),
            sequence_repeated: step == 0,
            sequence_reset: step > u32::MAX / 2,
        }
    }

    pub fn clock_name(&self) -> &'static str {
        match self.previous.map(|metadata| metadata.time) {
            Some(DrmEventTime::Monotonic(_)) => "monotonic",
            Some(DrmEventTime::Realtime(_)) => "realtime",
            None => "unavailable",
        }
    }
}

/// Read the same clock as the kernel timestamp, never subtract an Instant epoch.
pub(super) fn dispatch_delay(time: DrmEventTime) -> Option<Duration> {
    let now = match time {
        DrmEventTime::Monotonic(_) => {
            DrmEventTime::Monotonic(Clock::<Monotonic>::new().now().into())
        }
        DrmEventTime::Realtime(_) => DrmEventTime::Realtime(SystemTime::now()),
    };
    elapsed(time, now)
}

pub(super) fn elapsed(earlier: DrmEventTime, later: DrmEventTime) -> Option<Duration> {
    match (earlier, later) {
        (DrmEventTime::Monotonic(earlier), DrmEventTime::Monotonic(later)) => {
            later.checked_sub(earlier)
        }
        (DrmEventTime::Realtime(earlier), DrmEventTime::Realtime(later)) => {
            later.duration_since(earlier).ok()
        }
        _ => None,
    }
}
