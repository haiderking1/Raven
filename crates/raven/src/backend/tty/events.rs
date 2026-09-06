use super::TtyBackend;
use crate::state::State;
use smithay::backend::{
    drm::{DrmError, DrmEvent},
    session::{Event as SessionEvent, Session},
    udev::UdevEvent,
};
use std::{error::Error, io::ErrorKind, time::Instant};

fn with_backend(
    state: &mut State,
    action: impl FnOnce(&mut TtyBackend, &mut State) -> Result<(), Box<dyn Error>>,
) {
    let Some(mut backend) = state.backend.take() else {
        return;
    };
    if backend.failure.is_none() {
        if let Err(error) = action(&mut backend, state) {
            backend.fail(state, error);
        }
    }
    state.backend = Some(backend);
}

pub(super) fn session(event: SessionEvent, state: &mut State) {
    with_backend(state, |backend, state| {
        match event {
            SessionEvent::PauseSession => {
                backend.schedule.pause();
                if let Some(input) = &mut backend.input {
                    super::input_lifecycle::suspend(input, state);
                }
                if let Some(device) = &mut backend.device {
                    device.drm.pause();
                }
            }
            SessionEvent::ActivateSession => {
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
                backend.schedule.resume(Instant::now());
                // The timer renders after notifier dispatch. No GL work is done
                // while libseat is still delivering its activation callbacks.
            }
        }
        Ok(())
    });
}

pub(super) fn drm(event: DrmEvent, state: &mut State) {
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
                if device.crtc != crtc || !backend.schedule.presented(Instant::now()) {
                    return Ok(());
                }
                device.compositor.frame_submitted()?;
                state.send_frames(state.start_time.elapsed());
                repaint(backend, state)?;
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

pub(super) fn timer(state: &mut State) {
    with_backend(state, |backend, state| {
        if !backend.session.is_active() {
            return Ok(());
        }
        if backend.schedule.stalled(Instant::now()) {
            return Err("DRM pageflip did not complete within three seconds; stopping instead of reusing an in-flight buffer".into());
        }
        repaint(backend, state)
    });
}

fn repaint(backend: &mut TtyBackend, state: &mut State) -> Result<(), Box<dyn Error>> {
    let now = Instant::now();
    if !backend.session.is_active() || !backend.schedule.due(now) {
        return Ok(());
    }
    let device = backend
        .device
        .as_mut()
        .ok_or("DRM device missing while rendering")?;
    let queued = backend.scene.render(device, state)?;
    backend.schedule.rendered(queued, Instant::now());
    if !queued {
        // A client can request a callback without committing new pixels. There is
        // then no pageflip to wake it, so the idle timer provides refresh pacing.
        state.send_frames(state.start_time.elapsed());
    }
    Ok(())
}
