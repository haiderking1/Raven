mod recovery;

use super::{TtyBackend, presentation};
use crate::state::State;
use smithay::backend::drm::DrmEventMetadata;
use std::{error::Error, time::Instant};

pub(super) fn complete(
    backend: &mut TtyBackend,
    state: &mut State,
    metadata: Option<DrmEventMetadata>,
) -> Result<(), Box<dyn Error>> {
    if backend.schedule.pending().is_none() {
        return Ok(());
    }
    let completion = presentation::completion(metadata);
    let had_successor = backend.schedule.queued().is_some();
    let device = backend
        .device
        .as_mut()
        .ok_or("DRM device missing on pageflip")?;
    // Smithay may drop BOTH userdata values if submitting the successor fails.
    // Do not propagate that error until the actual completed frame is delivered.
    let submitted = device.compositor.frame_submitted();
    let finished = backend
        .schedule
        .presented(completion.instant, Instant::now())
        .ok_or("pageflip lost scheduler ownership")?;
    let matched = matches!(&submitted, Ok(Some(carrier)) if carrier.same_frame(&finished.feedback));
    if !matched && !(had_successor && submitted.is_err()) {
        return Err("DRM compositor and scheduler disagree on the completed frame".into());
    }
    let successor_submitted = had_successor && matched;
    let mut feedback = finished
        .feedback
        .take()
        .ok_or("presented frame missing feedback ownership")?;
    backend.presentation.complete(
        &mut feedback,
        metadata,
        &completion,
        device.output.current_mode().map_or(0, |mode| mode.refresh),
    );
    if let Some(timing) = &mut backend.timing {
        timing.presented(metadata);
        if successor_submitted {
            timing.kms_submitted();
        }
    }
    if let Some(timing) = &mut state.input_timing {
        timing.presented(metadata, successor_submitted);
    }
    match submitted {
        Ok(Some(carrier)) if carrier.same_frame(&finished.feedback) => Ok(()),
        Ok(_) => Err("DRM compositor and scheduler disagree on the completed frame".into()),
        Err(error) => {
            let rejected = backend.schedule.discard_pending();
            recovery::recover(backend, rejected, error)
        }
    }
}
