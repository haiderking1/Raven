use super::super::TtyBackend;
use crate::state::State;
use smithay::{
    backend::{
        allocator::{Buffer, dmabuf::Dmabuf},
        renderer::ImportDma,
        session::Session,
    },
    wayland::dmabuf::{DmabufGlobal, ImportNotifier},
};

impl TtyBackend {
    pub(crate) fn import_client_dmabuf(
        &mut self,
        global: DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        // Bound globals can outlive their advertisement. Never use EGL after a
        // fatal device error or while libseat has revoked device access.
        if self.failure.is_some()
            || !self.session.is_active()
            || !self
                .dmabuf
                .as_ref()
                .is_some_and(|registration| registration.global == global)
        {
            notifier.failed();
            return;
        }
        let Some(device) = self.device.as_mut().filter(|device| device.drm.is_active()) else {
            notifier.failed();
            return;
        };
        if !device.renderer.has_dmabuf_format(dmabuf.format()) {
            notifier.invalid_format();
            return;
        }
        // Smithay checks the params' dimensions, plane ordering, and bounds.
        // EGL/GLES must still validate the actual fds, layout and modifier.
        // Import is not sampling; acquire fences are checked on every attach
        // commit, including later reuse of the same wl_buffer.
        match device.renderer.import_dmabuf(&dmabuf, None) {
            Ok(_) => {
                // A disconnected client needs no reply. Smithay retains the
                // DMA-BUF in wl_buffer and the renderer caches its texture.
                let _ = notifier.successful::<State>();
            }
            Err(error) => {
                eprintln!("raven: DMA-BUF GLES import failed: {error}");
                // Sends failed for create, a protocol error for create_immed.
                notifier.failed();
            }
        }
    }
}
