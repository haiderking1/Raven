use std::time::{Duration, Instant};

const FLIP_TIMEOUT: Duration = Duration::from_secs(3);

/// Only one frame may be in flight. No-damage frames are retried by a refresh-rate
/// timer so clients waiting on frame callbacks can wake even on a static desktop.
pub(super) struct Schedule {
    active: bool,
    pending: Option<Instant>,
    next_frame: Instant,
    interval: Duration,
}

impl Schedule {
    pub fn new(refresh_millihz: i32, now: Instant) -> Self {
        let interval = if refresh_millihz > 0 {
            Duration::from_nanos(1_000_000_000_000 / refresh_millihz as u64)
        } else {
            Duration::from_millis(16)
        };
        Self {
            active: true,
            pending: None,
            next_frame: now,
            interval,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn due(&self, now: Instant) -> bool {
        self.active && self.pending.is_none() && now >= self.next_frame
    }

    pub fn stalled(&self, now: Instant) -> bool {
        self.active
            && self
                .pending
                .is_some_and(|submitted| now.duration_since(submitted) >= FLIP_TIMEOUT)
    }

    pub fn rendered(&mut self, queued: bool, now: Instant) {
        debug_assert!(self.active && self.pending.is_none());
        self.pending = queued.then_some(now);
        self.next_frame = now + self.interval;
    }

    pub fn presented(&mut self, now: Instant) -> bool {
        if !self.active || self.pending.take().is_none() {
            return false;
        }
        self.next_frame = now;
        true
    }

    pub fn pause(&mut self) {
        self.active = false;
        self.pending = None;
    }

    /// Call only after the DRM compositor's pending frames have also been cleared.
    pub fn resume(&mut self, now: Instant) {
        self.pending = None;
        self.next_frame = now;
        self.active = true;
    }
}

#[cfg(test)]
mod tests;
