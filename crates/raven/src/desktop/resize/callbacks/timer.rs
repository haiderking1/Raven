use crate::state::State;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use std::time::Duration;

impl State {
    fn resize_callback_interval(&self) -> Duration {
        let refresh = self
            .output
            .as_ref()
            .and_then(|output| output.current_mode())
            .map(|mode| mode.refresh)
            .filter(|refresh| *refresh > 0)
            .unwrap_or(60_000);
        Duration::from_secs_f64(1000.0 / f64::from(refresh))
    }

    pub(super) fn arm_resize_callbacks(&mut self) {
        if self.resize.callbacks.timer.is_some() || self.resize.callbacks.pending.is_empty() {
            return;
        }
        let Some(handle) = &self.resize.handle else {
            // Coordination itself is released by arm_resize_deadline when the
            // event source is absent. Keep these callbacks for normal delivery.
            self.cancel_resize_callbacks(None);
            return;
        };
        let interval = self.resize_callback_interval();
        match handle.insert_source(Timer::from_duration(interval), |_, _, state| {
            let now = state.start_time.elapsed();
            let interval = state.resize_callback_interval();
            state.resize.callbacks.pending.retain_mut(|pending| {
                let Ok(surface) = pending.surface.upgrade() else {
                    return false;
                };
                state.frame_callbacks.coordination(
                    &surface,
                    &mut pending.callbacks,
                    pending.queued,
                    now,
                    interval,
                );
                !pending.callbacks.is_empty()
            });
            if state.resize.callbacks.pending.is_empty() {
                state.resize.callbacks.timer = None;
                TimeoutAction::Drop
            } else {
                TimeoutAction::ToDuration(interval)
            }
        }) {
            Ok(token) => self.resize.callbacks.timer = Some(token),
            Err(error) => {
                eprintln!("raven: cannot register resize callback pacing: {error}");
                self.cancel_resize_callbacks(None);
                self.release_resize_transaction();
            }
        }
    }

    pub(crate) fn cancel_resize_callbacks(
        &mut self,
        surface: Option<&smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
    ) {
        use smithay::reexports::wayland_server::Resource;
        let cancelled = surface.map(Resource::downgrade);
        let pending = std::mem::take(&mut self.resize.callbacks.pending);
        for pending in pending {
            if cancelled
                .as_ref()
                .is_some_and(|surface| &pending.surface != surface && &pending.owner != surface)
            {
                self.resize.callbacks.pending.push(pending);
                continue;
            }
            if let Ok(surface) = pending.surface.upgrade() {
                smithay::wayland::compositor::with_states(&surface, |states| {
                    let mut attributes = states
                        .cached_state
                        .get::<smithay::wayland::compositor::SurfaceAttributes>();
                    let mut callbacks = pending.callbacks;
                    callbacks.append(&mut attributes.current().frame_callbacks);
                    attributes.current().frame_callbacks = callbacks;
                    drop(attributes);
                    self.frame_callbacks.hold_coordination(states, false);
                });
            }
        }
        if self.resize.callbacks.pending.is_empty()
            && let Some(token) = self.resize.callbacks.timer.take()
            && let Some(handle) = &self.resize.handle
        {
            handle.remove(token);
        }
    }
}
