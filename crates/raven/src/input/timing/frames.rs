use super::{InputTiming, snapshots::Pending};
use smithay::backend::drm::{DrmEventMetadata, DrmEventTime};
use std::time::Duration;

#[derive(Default)]
pub(super) struct Presentations {
    pub accepted: u64,
    pub without_snapshot: u64,
    pub ambiguous: u64,
    pub metadata_missing: u64,
    pub realtime: u64,
    pub monotonic: u64,
    pub zero: u64,
    pub future: u64,
    pub regressed: u64,
    pub clock_unavailable: u64,
    pub before_queue: u64,
    pub before_submission: u64,
}

impl InputTiming {
    /// Call once after successful changed-frame acceptance. With deferred=false,
    /// KMS submission succeeded and no frame was pending. With deferred=true,
    /// software accepted a successor behind exactly one pending KMS frame.
    /// Never call for empty renders, failed queues, retries, or redraw requests.
    /// Snapshots observations since the last acceptance, not client state.
    pub fn frame_queued(&mut self, deferred: bool) {
        let queued = self.monotonic();
        let observations = std::mem::take(&mut self.unqueued);
        let w = &mut self.window;
        w.queued = w.queued.saturating_add(1);
        w.deferred_queued = w.deferred_queued.saturating_add(u64::from(deferred));
        w.queue_age.record(observations, queued);
        let acceptance = self.snapshots.accept(observations, queued, deferred);
        w.queue_invariant_mismatch = w
            .queue_invariant_mismatch
            .saturating_add(u64::from(acceptance.invariant_mismatch));
        w.snapshots_discarded = w.snapshots_discarded.saturating_add(acceptance.discarded);
    }

    /// Only for the selected CRTC after the scheduler accepts its pending flip.
    /// Call after compositor.frame_submitted. successor_submitted is true ONLY
    /// when that call succeeded AND a software successor existed. Pass original
    /// metadata, never the scheduler's synthesized fallback time.
    pub fn presented(&mut self, metadata: Option<DrmEventMetadata>, successor_submitted: bool) {
        // This post-submission hook clock is also the successor's KMS boundary.
        let now = self.monotonic();
        let completion = self.snapshots.complete(now, successor_submitted);
        let w = &mut self.window;
        w.successor_invariant_mismatch = w
            .successor_invariant_mismatch
            .saturating_add(u64::from(completion.invariant_mismatch));
        if completion.successor_discarded {
            w.successor_submit_failed = w.successor_submit_failed.saturating_add(1);
            w.snapshots_discarded = w.snapshots_discarded.saturating_add(1);
        }
        if let Some((queued, submitted)) = completion.promotion {
            w.successor_promoted = w.successor_promoted.saturating_add(1);
            if let (Some(queued), Some(submitted)) = (queued, submitted) {
                if let Some(residence) = submitted.checked_sub(queued) {
                    w.software_residence.add(residence);
                } else {
                    w.phase_negative = w.phase_negative.saturating_add(1);
                }
            }
        }
        w.presentations.accepted = w.presentations.accepted.saturating_add(1);
        let time = self.presentation_time(metadata, now);
        if let (Some(time), Some(now)) = (time, now) {
            // presentation_time has rejected future timestamps.
            self.window.drm_dispatch.add(now - time);
        }
        match completion.presented {
            None => {
                let p = &mut self.window.presentations;
                p.without_snapshot = p.without_snapshot.saturating_add(1);
            }
            Some(Pending::Ambiguous) => {
                let p = &mut self.window.presentations;
                p.ambiguous = p.ambiguous.saturating_add(1);
            }
            Some(Pending::Snapshot(snapshot)) => {
                let p = &mut self.window.presentations;
                let time = time.filter(|&time| {
                    if snapshot.queued.is_some_and(|queued| time < queued) {
                        p.before_queue = p.before_queue.saturating_add(1);
                        false
                    } else if snapshot.submitted.is_some_and(|submitted| time < submitted) {
                        p.before_submission = p.before_submission.saturating_add(1);
                        false
                    } else {
                        true
                    }
                });
                self.window
                    .presentation_age
                    .record(snapshot.observations, time);
                if let (Some(time), Some(queued)) = (time, snapshot.queued) {
                    self.window.queue_to_presentation.add(time - queued);
                }
                if let (Some(time), Some(submitted)) = (time, snapshot.submitted) {
                    self.window.kms_to_presentation.add(time - submitted);
                }
            }
        }
    }

    fn presentation_time(
        &mut self,
        metadata: Option<DrmEventMetadata>,
        now: Option<Duration>,
    ) -> Option<Duration> {
        let p = &mut self.window.presentations;
        let Some(metadata) = metadata else {
            p.metadata_missing = p.metadata_missing.saturating_add(1);
            self.last_presentation = None;
            return None;
        };
        let time = match metadata.time {
            DrmEventTime::Realtime(_) => {
                p.realtime = p.realtime.saturating_add(1);
                self.last_presentation = None;
                return None;
            }
            DrmEventTime::Monotonic(time) => {
                p.monotonic = p.monotonic.saturating_add(1);
                time
            }
        };
        if time.is_zero() {
            p.zero = p.zero.saturating_add(1);
            self.last_presentation = None;
            return None;
        }
        let Some(now) = now else {
            p.clock_unavailable = p.clock_unavailable.saturating_add(1);
            self.last_presentation = None;
            return None;
        };
        if time > now {
            p.future = p.future.saturating_add(1);
            return None;
        }
        if self.last_presentation.is_some_and(|last| time < last) {
            p.regressed = p.regressed.saturating_add(1);
            return None;
        }
        self.last_presentation = Some(time);
        Some(time)
    }
}
