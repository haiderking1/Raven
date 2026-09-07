use super::{
    events::{Dispatch, KINDS},
    frames::Presentations,
    observations::Ages,
    samples::Samples,
};
use std::{fmt, time::Duration};

#[derive(Default)]
pub(super) struct Window {
    pub dispatch: [Dispatch; 10],
    pub clock_read_errors: u64,
    pub flush_attempt_age: Ages,
    pub queue_age: Ages,
    pub presentation_age: Ages,
    pub queued: u64,
    pub queue_overlap: u64,
    pub presentations: Presentations,
    pub queue_to_presentation: Samples,
    pub drm_dispatch: Samples,
    pub flushes: u64,
    pub flush_errors: u64,
    pub flush_duration: Samples,
    pub waits: u64,
    pub wait_errors: u64,
    pub wait_duration: Samples,
    pub phase_negative: u64,
}

/// One bounded report. Formatting and writing are the caller's responsibility.
pub struct Report {
    pub(super) span: Duration,
    pub(super) window: Window,
    pub(super) pending: bool,
    pub(super) unqueued: u64,
    pub(super) unflushed: u64,
}

impl Ages {
    fn format(&self, f: &mut fmt::Formatter<'_>, name: &str) -> fmt::Result {
        write!(
            f,
            " {name}_obs_batches={} {name}_obs={} {name}_valid_source_obs={} {name}_source_oldest_ms={} {name}_source_latest_ms={} {name}_dispatch_oldest_ms={} {name}_dispatch_latest_ms={} {name}_negative={}",
            self.batches,
            self.observations,
            self.valid_sources,
            self.source_oldest,
            self.source_latest,
            self.dispatch_oldest,
            self.dispatch_latest,
            self.negative
        )
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = &self.window;
        let p = &w.presentations;
        write!(
            f,
            "raven: input-timing span={:.3}s clock=CLOCK_MONOTONIC samples=n:min/mean/max scope=observations_not_client_response clock_read_errors={} queued={} queue_overlap={} pending={} unqueued_obs={} unflushed_obs={}",
            self.span.as_secs_f64(),
            w.clock_read_errors,
            w.queued,
            w.queue_overlap,
            self.pending,
            self.unqueued,
            self.unflushed
        )?;
        for (name, d) in KINDS.iter().zip(&w.dispatch) {
            if d.observed == 0 {
                continue;
            }
            write!(
                f,
                " {name}_observed={} {name}_dispatch_age_ms={} {name}_missing={} {name}_future={} {name}_clock_unavailable={}",
                d.observed, d.age, d.missing, d.future, d.clock_unavailable
            )?;
        }
        w.flush_attempt_age.format(f, "flush_attempt")?;
        w.queue_age.format(f, "queue")?;
        w.presentation_age.format(f, "presentation_snapshot")?;
        write!(
            f,
            " accepted_presentations={} presentation_without_snapshot={} presentation_ambiguous={} drm_metadata_missing={} drm_realtime={} drm_monotonic={} drm_zero={} drm_future={} drm_regressed={} drm_clock_unavailable={} drm_before_queue={} queue_return_to_drm_ms={} drm_dispatch_ms={} flushes={} flush_errors={} flush_ms={} waits={} wait_errors={} wait_ms={} phase_negative={}",
            p.accepted,
            p.without_snapshot,
            p.ambiguous,
            p.metadata_missing,
            p.realtime,
            p.monotonic,
            p.zero,
            p.future,
            p.regressed,
            p.clock_unavailable,
            p.before_queue,
            w.queue_to_presentation,
            w.drm_dispatch,
            w.flushes,
            w.flush_errors,
            w.flush_duration,
            w.waits,
            w.wait_errors,
            w.wait_duration,
            w.phase_negative
        )
    }
}
