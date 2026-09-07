use super::super::{TtyBackend, presentation::Frame, render::recovery};
use smithay::backend::drm::compositor::FrameError;
use std::{error::Error, io};

pub(super) fn recover<A, B, F>(
    backend: &mut TtyBackend,
    rejected: Option<Frame>,
    error: FrameError<A, B, F>,
) -> Result<(), Box<dyn Error>>
where
    A: Error + Send + Sync + 'static,
    B: Error + Send + Sync + 'static,
    F: Error + Send + Sync + 'static,
{
    let used_planes = rejected.as_ref().is_some_and(Frame::uses_planes);
    // This snapshot did not reach KMS. Discard its feedback, never substitute
    // the preceding frame's timestamp or attach it to freshly rendered content.
    drop(rejected);
    let device = backend
        .device
        .as_mut()
        .ok_or("DRM device missing during recovery")?;
    if !used_planes || !device.drm.is_atomic() || !recovery::plane_rejection(&error) {
        return Err(error.into());
    }
    let original = Some(error.to_string());
    if !device.drm.is_active() {
        return Err(recovery::failure(
            &original,
            io::Error::other("DRM device became inactive"),
        ));
    }
    device
        .validate_output()
        .map_err(|error| recovery::failure(&original, error))?;
    eprintln!(
        "raven: deferred plane submit rejected ({error}); scheduling one full-composition recovery"
    );
    device.plane_policy.suspend();
    device.compositor.reset_buffers();
    backend.deferred_recovery = original;
    backend.schedule.request_redraw();
    // Fresh scene assembly happens once at end of dispatch, not inside DRM IO.
    Ok(())
}
