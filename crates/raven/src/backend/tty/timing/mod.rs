mod flip;
mod report;
mod samples;
mod window;

#[cfg(test)]
mod tests;

use flip::Flips;
use report::Report;
use smithay::backend::drm::DrmEventMetadata;
use std::{
    ffi::OsStr,
    time::{Duration, Instant},
};
use window::Window;

const REPORT_INTERVAL: Duration = Duration::from_secs(2);

pub(super) struct Timing {
    refresh_millihz: i32,
    last_report: Instant,
    window: Window,
    flips: Flips,
}

impl Timing {
    pub fn from_env(refresh_millihz: i32, now: Instant) -> Option<Self> {
        if std::env::var_os("RAVEN_FRAME_TIMING").as_deref() != Some(OsStr::new("1")) {
            return None;
        }
        eprintln!(
            "raven: frame-timing enabled; reports every 2s; *_ms=min/mean/max; idle flips are not FPS loss"
        );
        Some(Self::new(refresh_millihz, now))
    }

    fn new(refresh_millihz: i32, now: Instant) -> Self {
        Self {
            refresh_millihz,
            last_report: now,
            window: Window::default(),
            flips: Flips::default(),
        }
    }

    pub fn reset(&mut self, now: Instant) {
        // VT inactivity and discarded pending flips must not become giant frame gaps.
        *self = Self::new(self.refresh_millihz, now);
    }

    pub fn rendered(&mut self, duration: Duration, queued: bool) {
        if queued {
            self.window.draws.add(duration);
        } else {
            self.window.empty.add(duration);
        }
    }

    pub fn timer_wakeup(&mut self, deadline: Option<Instant>, now: Instant) {
        let Some(deadline) = deadline else {
            return;
        };
        if let Some(lateness) = now.checked_duration_since(deadline) {
            self.window.timer_lateness.add(lateness);
        } else {
            self.window.timer_early += 1;
        }
    }

    pub fn presented(&mut self, metadata: Option<DrmEventMetadata>) {
        self.window.flips += 1;
        let observation = self.flips.observe(metadata);
        if let Some(interval) = observation.interval {
            self.window.flip_intervals.add(interval);
        }
        if let Some(step) = observation.step {
            self.window.steps[step.min(3) as usize - 1] += 1;
        }
        self.window.clock_discontinuities += u64::from(observation.clock_discontinuity);
        self.window.repeated_sequences += u64::from(observation.sequence_repeated);
        self.window.reset_sequences += u64::from(observation.sequence_reset);
        self.window.last_sequence = metadata.map(|metadata| metadata.sequence);
        if let Some(metadata) = metadata {
            if let Some(delay) = flip::dispatch_delay(metadata.time) {
                self.window.dispatch_delay.add(delay);
            }
        } else {
            self.window.missing_metadata += 1;
        }
    }

    pub fn deadline(&self) -> Instant {
        self.last_report + REPORT_INTERVAL
    }

    pub fn report(&mut self, now: Instant) -> Option<Report> {
        let span = now.duration_since(self.last_report);
        if span < REPORT_INTERVAL {
            return None;
        }
        self.last_report = now;
        Some(Report {
            span,
            refresh_millihz: self.refresh_millihz,
            clock: self.flips.clock_name(),
            window: std::mem::take(&mut self.window),
        })
    }
}
