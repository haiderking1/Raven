//! Linux DMA-BUF dispatch. The TTY registration owns the advertised global.
pub(crate) mod acquire;
mod blocker;

use crate::state::State;
use smithay::{
    backend::allocator::dmabuf::Dmabuf,
    delegate_dmabuf,
    wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
};

impl DmabufHandler for State {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        // The backend owns its delegate so dropping it can destroy the global
        // before EGL/DRM. Without a backend, no DMA-BUF global is advertised.
        self.backend
            .as_mut()
            .and_then(|backend| backend.dmabuf_state())
            .unwrap_or(&mut self.dmabuf_state)
    }

    fn dmabuf_imported(&mut self, global: &DmabufGlobal, dmabuf: Dmabuf, notifier: ImportNotifier) {
        match self.backend.as_mut() {
            Some(backend) => backend.import_client_dmabuf(*global, dmabuf, notifier),
            None => notifier.failed(),
        }
    }
}

delegate_dmabuf!(State);
