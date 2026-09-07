mod frame;

use super::{TtyBackend, events::with_backend};
use crate::state::State;
use smithay::backend::session::Session;
use std::time::Instant;

impl TtyBackend {
    /// Run once after all ready sources and desktop reconciliation, before flushing clients.
    pub(crate) fn dispatch(state: &mut State) {
        with_backend(state, |backend, state| {
            if state.take_redraw_request() {
                backend.schedule.request_redraw();
            }
            if !backend.schedule.active() || !backend.session.is_active() {
                backend.sources.arm(None)?;
                return Ok(());
            }
            if backend.schedule.stalled(Instant::now()) {
                return Err("DRM pageflip did not complete within three seconds; stopping instead of reusing an in-flight buffer".into());
            }
            if backend.schedule.render_due() {
                frame::render(backend, state)?;
            }
            // An empty render after a long idle may have callbacks already due.
            send_callbacks(backend, state);
            if let Some(timing) = &mut backend.timing
                && let Some(report) = timing.report(Instant::now())
            {
                eprintln!("{report}");
            }
            if let Some(timing) = &mut state.input_timing
                && let Some(report) = timing.report(Instant::now())
            {
                eprintln!("{report}");
            }
            let background_deadline = state.background_frame_deadline(state.start_time.elapsed());
            let deadline = backend
                .schedule
                .deadline()
                .into_iter()
                .chain(backend.timing.as_ref().map(|timing| timing.deadline()))
                .chain(background_deadline)
                .chain(state.input_timing.as_ref().map(|timing| timing.deadline()))
                .min();
            backend.sources.arm(deadline)?;
            Ok(())
        });
    }
}

fn send_callbacks(backend: &mut TtyBackend, state: &State) {
    let now = Instant::now();
    if backend.schedule.callbacks_due(now) {
        if backend.schedule.callbacks_advance_cycle(now) {
            state.send_frames(state.start_time.elapsed());
        } else {
            state.resend_frames(state.start_time.elapsed());
        }
        backend.schedule.callbacks_sent(now);
    }
}
