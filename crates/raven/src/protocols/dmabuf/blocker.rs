use smithay::{
    backend::allocator::dmabuf::DmabufBlocker,
    reexports::wayland_server::{Weak, protocol::wl_surface::WlSurface},
    wayland::compositor::{Blocker, BlockerState},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// A removed source must not leave a transaction blocked forever. A destroyed
/// surface no longer needs acquisition, but its live synchronized siblings do.
pub(super) struct AcquireBlocker {
    pub fence: DmabufBlocker,
    pub surface: Weak<WlSurface>,
    pub cancelled: Arc<AtomicBool>,
}

impl Blocker for AcquireBlocker {
    fn state(&self) -> BlockerState {
        if self.surface.upgrade().is_err() {
            BlockerState::Released
        } else if self.cancelled.load(Ordering::SeqCst) {
            BlockerState::Cancelled
        } else {
            self.fence.state()
        }
    }
}
