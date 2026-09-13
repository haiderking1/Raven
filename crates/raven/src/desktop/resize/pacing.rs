use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::calloop::{
        RegistrationToken,
        timer::{TimeoutAction, Timer},
    },
};
use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct Pacing {
    last: Option<(Window, Instant)>,
    timer: Option<RegistrationToken>,
}

impl State {
    pub(crate) fn reset_floating_resize_pacing(&mut self) {
        if let Some(token) = self.resize.pacing.timer.take()
            && let Some(handle) = &self.resize.handle
        {
            handle.remove(token);
        }
        self.resize.pacing.last = None;
    }

    /// Coalesce floating geometry/configures to the output rate without waiting
    /// for an ACK. A one-shot timer delivers the last motion even if input stops.
    pub(crate) fn defer_floating_resize(&mut self, window: &Window) -> bool {
        let now = Instant::now();
        let refresh = self
            .output
            .as_ref()
            .and_then(|output| output.current_mode())
            .filter(|mode| mode.refresh > 0)
            .map_or(60_000, |mode| mode.refresh);
        let interval = Duration::from_secs_f64(1000.0 / f64::from(refresh));
        if let Some((previous, last)) = &self.resize.pacing.last
            && previous == window
            && now < *last + interval
        {
            if self.resize.pacing.timer.is_some() {
                return true;
            }
            if let Some(handle) = &self.resize.handle {
                match handle.insert_source(Timer::from_deadline(*last + interval), |_, _, state| {
                    state.resize.pacing.timer = None;
                    state.reconcile_window_drag();
                    state.apply_window_resize();
                    TimeoutAction::Drop
                }) {
                    Ok(token) => {
                        self.resize.pacing.timer = Some(token);
                        return true;
                    }
                    Err(error) => eprintln!("raven: cannot pace floating resize: {error}"),
                }
            }
        }
        if let Some(token) = self.resize.pacing.timer.take()
            && let Some(handle) = &self.resize.handle
        {
            handle.remove(token);
        }
        self.resize.pacing.last = Some((window.clone(), now));
        false
    }
}
