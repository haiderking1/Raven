//! Opt-in compositor boundary diagnostics. These are observations, not causal
//! client-response or input-to-photon measurements. Runtime owns all hook calls.

mod boundaries;
mod clock;
mod events;
mod frames;
mod observations;
mod report;
mod samples;

pub use report::Report;

use observations::Observations;
use report::Window;
use std::time::{Duration, Instant};

const REPORT_INTERVAL: Duration = Duration::from_secs(2);

/// A single explicit flush attempt, not acknowledgement by any client.
pub struct FlushBoundary(boundaries::Mark);
/// A single blocking GPU synchronization wait.
pub struct WaitBoundary(boundaries::Mark);

pub struct InputTiming {
    since: Instant,
    generation: u64,
    window: Window,
    unflushed: Observations,
    unqueued: Observations,
    pending: Option<frames::Pending>,
    last_presentation: Option<Duration>,
}

impl InputTiming {
    /// Only the exact value "1" enables diagnostics. Disabled initialization
    /// does not read either clock and allocates no timing state.
    pub fn from_env() -> Option<Self> {
        if std::env::var_os("RAVEN_INPUT_TIMING").as_deref() != Some(std::ffi::OsStr::new("1")) {
            return None;
        }
        Some(Self::new(Instant::now(), 0))
    }

    fn new(now: Instant, generation: u64) -> Self {
        Self {
            since: now,
            generation,
            window: Window::default(),
            unflushed: Observations::default(),
            unqueued: Observations::default(),
            pending: None,
            last_presentation: None,
        }
    }

    /// Call on both VT pause and successful resume. Discards partial reports,
    /// input batches, the pending snapshot, and outstanding phase tokens.
    pub fn reset(&mut self, now: Instant) {
        *self = Self::new(now, self.generation.wrapping_add(1));
    }

    /// Merge with the existing backend deadline only while the session is active.
    pub fn deadline(&self) -> Instant {
        self.since + REPORT_INTERVAL
    }

    /// Return at most one batch, with no logging or catch-up report loop.
    /// Pending frame and input snapshots survive reporting, but not reset.
    pub fn report(&mut self, now: Instant) -> Option<Report> {
        if now < self.deadline() {
            return None;
        }
        let report = Report {
            span: now.duration_since(self.since),
            window: std::mem::take(&mut self.window),
            pending: self.pending.is_some(),
            unqueued: self.unqueued.count,
            unflushed: self.unflushed.count,
        };
        self.since = now;
        Some(report)
    }

    fn monotonic(&mut self) -> Option<Duration> {
        let now = clock::now();
        if now.is_none() {
            self.window.clock_read_errors = self.window.clock_read_errors.saturating_add(1);
        }
        now
    }
}
