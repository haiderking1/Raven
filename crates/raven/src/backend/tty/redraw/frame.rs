use super::super::{TtyBackend, presentation::Frame, render::recovery};
use crate::state::State;
use std::{error::Error, io, time::Instant};

pub(super) fn render(backend: &mut TtyBackend, state: &mut State) -> Result<(), Box<dyn Error>> {
    let device = backend
        .device
        .as_mut()
        .ok_or("DRM device missing while rendering")?;
    let deferred = backend.schedule.pending().is_some();
    let started = (device.gpu_time.is_some() || backend.timing.is_some()).then(Instant::now);
    let gpu = if let Some(timer) = &mut device.gpu_time {
        timer.sample(&mut device.renderer)?
    } else {
        None
    };
    let mut outcome = backend
        .scene
        .render(device, state, deferred)
        .map_err(|error| recovery::failure(&backend.deferred_recovery, error))?;
    if backend.deferred_recovery.is_some() {
        if !outcome.queued {
            return Err(recovery::failure(
                &backend.deferred_recovery,
                io::Error::other("deferred composition recovery produced no frame"),
            ));
        }
        backend.deferred_recovery = None;
        outcome.recovered = true;
    }
    let queued = outcome.queued;
    if let Some(timing) = &mut backend.timing {
        timing.planes(&outcome);
        timing.queue_placement(queued, deferred);
    }
    let ticket = if queued {
        Some(Frame {
            feedback: outcome
                .feedback
                .take()
                .ok_or("queued frame missing feedback carrier")?,
            primary_scanout: outcome.primary_scanout,
            hardware_cursor: outcome.hardware_cursor,
        })
    } else {
        None
    };
    let finished = Instant::now();
    backend.schedule.rendered(ticket, finished);
    // Admission used the old budget. Sampling must not move that deadline and
    // make an already-accepted frame disappear from scheduler ownership.
    if let Some(started) = started {
        let cpu = finished.duration_since(started);
        if queued {
            backend.schedule.observe_render(cpu);
        }
        if let Some(timing) = &mut backend.timing {
            timing.rendered(cpu, queued);
            timing.gpu_sample(gpu);
        }
    }
    if let Some(gpu) = gpu {
        backend.schedule.observe_gpu(gpu);
    }
    Ok(())
}
