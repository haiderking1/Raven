//! Direct KMS/GBM/GLES backend for one GPU and one output.
//!
//! Call TtyBackend::install before dispatching clients. It owns every backend
//! calloop registration and replaces State's bootstrap seat if libseat names a
//! different seat. Runtime device failures stop State's loop signal; inspect
//! failure() after the loop to report a nonzero exit status.
mod device;
mod dmabuf;
mod events;
mod input_lifecycle;
mod install;
mod presentation;
mod redraw;
mod render;
mod schedule;
mod session;
mod sources;
mod timing;
mod wake;

use crate::state::State;
use device::Device;
use render::Scene;
use schedule::Schedule;
use smithay::{
    backend::session::{Session, libseat::LibSeatSession},
    reexports::{
        calloop::LoopHandle,
        input::Libinput,
        wayland_server::{DisplayHandle, backend::GlobalId},
    },
};
use sources::Sources;
use std::error::Error;

pub struct TtyBackend {
    dmabuf: Option<dmabuf::Registration>,
    device: Option<Device>,
    session: LibSeatSession,
    input: Option<Libinput>,
    scene: Scene,
    schedule: Schedule,
    presentation: presentation::Presentation,
    timing: Option<timing::Timing>,
    sources: Sources,
    display_handle: DisplayHandle,
    output_global: Option<GlobalId>,
    failure: Option<String>,
}

impl TtyBackend {
    /// Open the active libseat session and install DRM, udev, libinput, session,
    /// and deadline timer sources. Does not run a nested event loop.
    pub fn install(
        state: &mut State,
        handle: LoopHandle<'static, State>,
    ) -> Result<(), Box<dyn Error>> {
        install::install(state, handle)
    }

    /// Request a VT through libseat, never through a privileged tty ioctl.
    pub fn change_vt(&mut self, vt: i32) -> Result<(), Box<dyn Error>> {
        if !(1..=63).contains(&vt) {
            return Err("VT number must be between 1 and 63".into());
        }
        if self.failure.is_some() || !self.session.is_active() {
            return Err("cannot switch VT while Raven's seat is inactive or stopped".into());
        }
        self.session.change_vt(vt)?;
        Ok(())
    }

    pub fn seat_name(&self) -> String {
        self.session.seat()
    }

    /// Fatal runtime error, if the backend requested event-loop termination.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    fn fail(&mut self, state: &State, error: impl std::fmt::Display) {
        if self.failure.is_none() {
            self.failure = Some(error.to_string());
        }
        self.schedule.pause();
        if let Some(input) = &mut self.input {
            input.suspend();
        }
        if let Some(registration) = &mut self.dmabuf {
            registration.disable();
        }
        // Do not issue any more device ioctls, including DRM restoration on drop,
        // after an unplug or a fatal access error.
        if let Some(device) = &mut self.device {
            device.drm.pause();
        }
        state.loop_signal.stop();
    }
}

impl Drop for TtyBackend {
    fn drop(&mut self) {
        self.schedule.pause();
        if let Some(input) = &mut self.input {
            input.suspend();
        }
        // Remove client readiness fds and globals before dropping EGL/DRM.
        self.dmabuf.take();
        self.sources.remove_devices();
        // Remove the notifier's DRM fd reference before closing the session fd.
        if let Some(mut device) = self.device.take() {
            if device.drm.is_active() && self.session.is_active() {
                if let Err(error) = device.compositor.clear() {
                    eprintln!("raven: could not clear output at shutdown: {error}");
                    device.drm.pause();
                }
            } else {
                device.drm.pause();
            }
            drop(device);
        }
        self.input.take();
        // LibSeatSession holds a weak reference. Keep its notifier alive until
        // every libinput and DRM close_device call has completed.
        self.sources.remove_session();
        if let Some(global) = self.output_global.take() {
            self.display_handle.remove_global::<State>(global);
        }
    }
}
