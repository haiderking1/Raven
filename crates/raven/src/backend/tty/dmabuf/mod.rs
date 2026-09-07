//! DMA-BUF imports, implicit acquire readiness, and per-surface allocation advice.
mod delivery;
mod feedback;
pub(super) mod identity;
pub(super) use delivery::FeedbackDelivery;
pub(super) use feedback::Feedback;
mod import;
mod pending;
mod registration;

pub(super) use registration::Registration;

use super::TtyBackend;
use crate::state::State;
use smithay::{
    backend::allocator::dmabuf::{Dmabuf, DmabufSource},
    reexports::wayland_server::backend::ObjectId,
    wayland::dmabuf::DmabufState,
};
use std::{
    io,
    sync::{Arc, atomic::AtomicBool},
};

impl TtyBackend {
    pub(crate) fn dmabuf_state(&mut self) -> Option<&mut DmabufState> {
        self.dmabuf
            .as_mut()
            .map(|registration| &mut registration.state)
    }

    /// Readiness monitoring remains available while the seat is paused. No GL
    /// work happens here, and applied commits render only after backend resume.
    pub(crate) fn wait_for_dmabuf(
        &mut self,
        source: DmabufSource,
        surface: ObjectId,
        cancelled: Arc<AtomicBool>,
        callback: impl FnMut(&mut Dmabuf, &mut State) -> io::Result<()> + 'static,
    ) -> io::Result<()> {
        let registration = self
            .dmabuf
            .as_mut()
            .filter(|_| self.failure.is_none())
            .ok_or_else(|| io::Error::other("DMA-BUF backend is unavailable"))?;
        registration
            .pending
            .insert(source, surface, cancelled, callback)
    }

    pub(crate) fn cancel_dmabuf_surface(&mut self, surface: &ObjectId) {
        if let Some(registration) = &mut self.dmabuf {
            registration.pending.remove_surface(surface);
        }
    }
}
