use super::super::super::TtyBackend;
use crate::state::State;
use smithay::{
    backend::session::Session, reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::Serial,
};

impl TtyBackend {
    /// The pre-commit caller has not installed the replacement surface state.
    /// Borrow the backend only for this copy; never dispatch clients reentrantly.
    pub(crate) fn capture_resize_animation(state: &mut State, root: &WlSurface, serial: Serial) {
        let Some(mut backend) = state.backend.take() else {
            return;
        };
        if backend.failure.is_none() && backend.session.is_active() && backend.schedule.active() {
            if let Some(device) = backend.device.as_mut() {
                match backend.scene.animations.capture(
                    &mut device.renderer,
                    state,
                    &device.output,
                    root,
                    serial,
                ) {
                    Ok(true) => state.request_redraw(),
                    Ok(false) => state.request_redraw(),
                    Err(error) => {
                        backend.scene.animations.remove(root);
                        state.request_redraw();
                        eprintln!("raven: resize snapshot cancelled: {error}");
                    }
                }
            }
        }
        state.backend = Some(backend);
    }

    pub(crate) fn cancel_resize_animations(&mut self) {
        self.scene.animations.clear();
    }
    pub(crate) fn cancel_resize_surface(&mut self, root: &WlSurface) {
        self.scene.animations.remove(root);
    }
}
