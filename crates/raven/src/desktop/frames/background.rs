use super::cycle::SurfaceFrames;
use crate::state::State;
use smithay::wayland::compositor::SurfaceAttributes;
use std::time::{Duration, Instant};

const OCCLUDED_INTERVAL: Duration = Duration::from_millis(250);

impl State {
    /// Wake occluded mapped clients at most four times a second, only while
    /// they have committed callbacks. This never requests a compositor repaint.
    pub(crate) fn background_frame_deadline(&self, now: Duration) -> Option<Instant> {
        if !self.frame_callbacks.occluded.get() {
            return None;
        }
        let output = self.output.as_ref()?;
        let mut next = None;
        self.with_frame_surfaces(|_, states| {
            let frames = states.data_map.get_or_insert(SurfaceFrames::default);
            let mut frames = frames.0.lock().expect("surface frame state poisoned");
            if frames.coordination_pending {
                return;
            }
            if !frames
                .visibility
                .as_ref()
                .is_some_and(|(owner, visible)| owner == output && !visible)
            {
                return;
            }
            let mut attributes = states.cached_state.get::<SurfaceAttributes>();
            let callbacks = &mut attributes.current().frame_callbacks;
            if callbacks.is_empty() {
                return;
            }
            let due = frames.last_time.unwrap_or(now) + OCCLUDED_INTERVAL;
            // Remember the first pending request even if this surface has never
            // received a visible callback. Otherwise its deadline would drift.
            frames.last_time.get_or_insert(now);
            if now >= due {
                for callback in callbacks.drain(..) {
                    callback.done(now.as_millis() as u32);
                }
                frames.last_time = Some(now);
                frames.last_cycle = Some(self.frame_callbacks.cycle.get());
            } else {
                let deadline = self.start_time + due;
                next = Some(next.map_or(deadline, |old: Instant| old.min(deadline)));
            }
        });
        next
    }
}
