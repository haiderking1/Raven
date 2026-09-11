use super::cycle::{FrameCallbacks, SurfaceFrames};
use smithay::{
    reexports::wayland_server::protocol::{wl_callback::WlCallback, wl_surface::WlSurface},
    wayland::compositor::{SurfaceAttributes, SurfaceData, with_states},
};
use std::time::Duration;

impl FrameCallbacks {
    pub(crate) fn hold_coordination(&self, states: &SurfaceData, held: bool) {
        states
            .data_map
            .get_or_insert(SurfaceFrames::default)
            .0
            .lock()
            .expect("surface frame state poisoned")
            .coordination_pending = held;
    }

    /// Callback completion is permission to produce another frame, not evidence
    /// that the held buffer was acquired, released, or presented.
    pub(crate) fn coordination(
        &self,
        surface: &WlSurface,
        callbacks: &mut Vec<WlCallback>,
        queued: Duration,
        now: Duration,
        interval: Duration,
    ) {
        with_states(surface, |states| {
            let frames = states.data_map.get_or_insert(SurfaceFrames::default);
            let mut frames = frames.0.lock().expect("surface frame state poisoned");
            if now < frames.last_time.unwrap_or(queued) + interval {
                return;
            }
            for callback in callbacks.drain(..) {
                callback.done(now.as_millis() as u32);
            }
            // State may have applied since extraction. These callbacks follow
            // the held requests and can share this opportunity, in order.
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(now.as_millis() as u32);
            }
            frames.coordination_pending = false;
            frames.last_time = Some(now);
            frames.last_cycle = Some(self.cycle.get());
            frames.coordination_until = Some(now + interval);
        });
    }
}
