use super::TtyBackend;
use crate::state::State;
use smithay::backend::{
    drm::{DrmError, DrmEvent, DrmEventMetadata},
    session::{Event as SessionEvent, Session},
    udev::UdevEvent,
};
use std::{error::Error, io::ErrorKind, time::Instant};

pub(super) fn with_backend(
    state: &mut State,
    action: impl FnOnce(&mut TtyBackend, &mut State) -> Result<(), Box<dyn Error>>,
) {
    let Some(mut backend) = state.backend.take() else {
        return;
    };
    if backend.failure.is_none() {
        if let Err(error) = action(&mut backend, state) {
            state.cancel_resize_transactions();
            backend.fail(state, error);
        }
    }
    state.backend = Some(backend);
}

pub(super) fn session(event: SessionEvent, state: &mut State) {
    with_backend(state, |backend, state| {
        match event {
            SessionEvent::PauseSession => {
                state.animations.clear_intents();
                state.suspend_resize_transactions();
                backend.pause_frames();
                if let Some(input) = &mut backend.input {
                    super::input_lifecycle::suspend(input, state);
                }
                if let Some(device) = &mut backend.device {
                    device.drm.pause();
                }
                if let Some(timing) = &mut state.input_timing {
                    timing.reset(Instant::now());
                }
            }
            SessionEvent::ActivateSession => {
                state.animations.clear_intents();
                if backend.schedule.active() {
                    return Ok(());
                }
                let device = backend
                    .device
                    .as_mut()
                    .ok_or("DRM device missing on resume")?;
                device.resume()?;
                backend
                    .input
                    .as_mut()
                    .ok_or("libinput missing on resume")?
                    .resume()
                    .map_err(|_| "libinput failed to resume the seat")?;
                state.resume_resize_transactions();
                state.resume_pointer_capture();
                let now = Instant::now();
                backend.schedule.resume(now);
                if let Some(timing) = &mut state.input_timing {
                    timing.reset(now);
                }
                if let Some(timing) = &mut backend.timing {
                    timing.reset(now);
                }
                // End-of-dispatch rendering keeps GL outside libseat callbacks.
            }
        }
        Ok(())
    });
}

pub(super) fn drm(event: DrmEvent, metadata: Option<DrmEventMetadata>, state: &mut State) {
    with_backend(state, |backend, state| {
        if !backend.schedule.active() || !backend.session.is_active() {
            return Ok(());
        }
        match event {
            DrmEvent::VBlank(crtc) => {
                let device = backend
                    .device
                    .as_mut()
                    .ok_or("DRM device missing on pageflip")?;
                if device.crtc != crtc {
                    return Ok(());
                }
                super::pageflip::complete(backend, state, metadata)?;
                // Callbacks and any requested repaint run after the whole event batch.
            }
            // Resume can drain a fd that calloop already marked readable in
            // this dispatch batch. Its notifier then legitimately sees EAGAIN.
            DrmEvent::Error(DrmError::Access(error))
                if matches!(
                    error.source.kind(),
                    ErrorKind::WouldBlock | ErrorKind::Interrupted
                ) => {}
            DrmEvent::Error(error) => return Err(error.into()),
        }
        Ok(())
    });
}

pub(super) fn udev(event: UdevEvent, state: &mut State) {
    with_backend(state, |backend, _| {
        let device = backend
            .device
            .as_mut()
            .ok_or("DRM device missing on udev event")?;
        match event {
            UdevEvent::Removed { device_id } if device_id == device.drm.device_id() => {
                return Err("the selected DRM GPU was removed; Raven is stopping".into());
            }
            UdevEvent::Changed { device_id } if device_id == device.drm.device_id() => {
                // DRM control traits bypass Smithay's active flag. Reprobe after
                // activation instead of making ioctls against a paused device.
                if backend.schedule.active() && backend.session.is_active() {
                    device.validate_output()?;
                }
            }
            // A second GPU/output is intentionally not added to this single-output session.
            _ => {}
        }
        Ok(())
    });
}

pub(super) fn timer(deadline: Instant, state: &mut State) {
    if let Some(backend) = &mut state.backend
        && backend.schedule.active()
        && let Some(timing) = &mut backend.timing
    {
        timing.timer_wakeup(Some(deadline), Instant::now());
    }
}
