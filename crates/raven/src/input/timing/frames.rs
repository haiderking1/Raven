use super::{InputTiming, observations::Observations};
use smithay::backend::drm::{DrmEventMetadata, DrmEventTime};
use std::time::Duration;

pub(super) enum Pending {
    Snapshot {
        observations: Observations,
        queued: Option<Duration>,
    },
    /// A broken hook invariant must not silently associate a flip with a frame.
    Ambiguous,
}

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
}

impl InputTiming {
    /// Call exactly once after a successful changed-frame queue. Never call for
    /// an empty render, failed queue, retry attempt, or mere redraw request.
    /// Snapshots observations since the last successful queue, not client state.
    pub fn frame_queued(&mut self) {
        let queued = self.monotonic();
        let observations = std::mem::take(&mut self.unqueued);
        self.window.queued = self.window.queued.saturating_add(1);
        self.window.queue_age.record(observations, queued);
        if self.pending.is_some() {
            self.window.queue_overlap = self.window.queue_overlap.saturating_add(1);
            self.pending = Some(Pending::Ambiguous);
        } else {
            self.pending = Some(Pending::Snapshot {
                observations,
                queued,
            });
        }
    }

    /// Only for the selected CRTC after the scheduler accepts its pending flip.
    /// Pass original metadata, never the scheduler's synthesized fallback time.
    pub fn presented(&mut self, metadata: Option<DrmEventMetadata>) {
        let now = self.monotonic();
        self.window.presentations.accepted = self.window.presentations.accepted.saturating_add(1);
        let time = self.presentation_time(metadata, now);
        if let (Some(time), Some(now)) = (time, now) {
            // presentation_time has rejected future timestamps.
            self.window.drm_dispatch.add(now - time);
        }
        match self.pending.take() {
            None => {
                let p = &mut self.window.presentations;
                p.without_snapshot = p.without_snapshot.saturating_add(1);
            }
            Some(Pending::Ambiguous) => {
                let p = &mut self.window.presentations;
                p.ambiguous = p.ambiguous.saturating_add(1);
            }
            Some(Pending::Snapshot {
                observations,
                queued,
            }) => {
                let time = match (time, queued) {
                    (Some(time), Some(queued)) if time < queued => {
                        let p = &mut self.window.presentations;
                        p.before_queue = p.before_queue.saturating_add(1);
                        None
                    }
                    (time, _) => time,
                };
                self.window.presentation_age.record(observations, time);
                if let (Some(time), Some(queued)) = (time, queued) {
                    self.window.queue_to_presentation.add(time - queued);
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
