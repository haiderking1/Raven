use super::{FlushBoundary, InputTiming, WaitBoundary};
use std::time::Duration;

pub(super) struct Mark {
    time: Option<Duration>,
    generation: u64,
}

impl InputTiming {
    pub fn before_flush(&mut self) -> FlushBoundary {
        let time = self.monotonic();
        self.window
            .flush_attempt_age
            .record(std::mem::take(&mut self.unflushed), time);
        FlushBoundary(Mark {
            time,
            generation: self.generation,
        })
    }

    /// Call even on failure, before propagating the original flush result.
    pub fn after_flush(&mut self, boundary: FlushBoundary, succeeded: bool) {
        if boundary.0.generation != self.generation {
            return;
        }
        let elapsed = self.elapsed(boundary.0);
        let w = &mut self.window;
        w.flushes = w.flushes.saturating_add(1);
        if !succeeded {
            w.flush_errors = w.flush_errors.saturating_add(1);
        }
        if let Some(elapsed) = elapsed {
            w.flush_duration.add(elapsed);
        }
    }

    pub fn before_wait(&mut self) -> WaitBoundary {
        WaitBoundary(Mark {
            time: self.monotonic(),
            generation: self.generation,
        })
    }

    /// Wrap only the actual sync.wait call, not render construction or queueing.
    pub fn after_wait(&mut self, boundary: WaitBoundary, succeeded: bool) {
        if boundary.0.generation != self.generation {
            return;
        }
        let elapsed = self.elapsed(boundary.0);
        let w = &mut self.window;
        w.waits = w.waits.saturating_add(1);
        if !succeeded {
            w.wait_errors = w.wait_errors.saturating_add(1);
        }
        if let Some(elapsed) = elapsed {
            w.wait_duration.add(elapsed);
        }
    }

    fn elapsed(&mut self, mark: Mark) -> Option<Duration> {
        let end = self.monotonic()?;
        let elapsed = end.checked_sub(mark.time?);
        if elapsed.is_none() {
            self.window.phase_negative = self.window.phase_negative.saturating_add(1);
        }
        elapsed
    }
}
