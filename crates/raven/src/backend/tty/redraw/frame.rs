use super::super::TtyBackend;
use crate::state::State;
use std::{error::Error, time::Instant};

pub(super) fn render(backend: &mut TtyBackend, state: &mut State) -> Result<(), Box<dyn Error>> {
    let device = backend
        .device
        .as_mut()
        .ok_or("DRM device missing while rendering")?;
    let started = backend.timing.as_ref().map(|_| Instant::now());
    let outcome = backend.scene.render(device, state)?;
    let queued = outcome.queued;
    let finished = Instant::now();
    backend.schedule.rendered(queued, finished);
    if let Some(timing) = &mut backend.timing
        && let Some(started) = started
    {
        timing.rendered(finished.duration_since(started), queued);
        timing.planes(&outcome);
    }
    Ok(())
}
