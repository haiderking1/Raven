use std::time::{Duration, Instant};

/// A fixed-refresh grid anchored to the last kernel presentation timestamp.
#[derive(Debug)]
pub(super) struct FrameClock {
    interval: Duration,
    anchor: Instant,
}

impl FrameClock {
    pub fn new(refresh_millihz: i32, now: Instant) -> Self {
        let refresh = if refresh_millihz > 0 {
            refresh_millihz
        } else {
            60_000
        };
        Self {
            interval: Duration::from_nanos(1_000_000_000_000 / refresh as u64),
            anchor: now,
        }
    }

    pub fn presented(&mut self, time: Instant) {
        if time > self.anchor {
            self.anchor = time;
        }
    }

    pub fn reset(&mut self, now: Instant) {
        self.anchor = now;
    }

    pub fn next(&self, now: Instant) -> Instant {
        if now < self.anchor {
            return self.anchor;
        }
        let phase = now.duration_since(self.anchor).as_nanos() % self.interval.as_nanos();
        now + self.interval - Duration::from_nanos(phase as u64)
    }
}
