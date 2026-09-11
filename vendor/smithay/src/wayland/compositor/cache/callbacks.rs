use super::CachedState;
use crate::wayland::compositor::SurfaceAttributes;
use wayland_server::protocol::wl_callback::WlCallback;

impl CachedState<SurfaceAttributes> {
    /// Remove frame callbacks from committed snapshots without applying state.
    ///
    /// Pending requests and current state are untouched. Callers must respect
    /// synchronized subsurface transaction boundaries and pace callback delivery.
    /// This does not signal buffer readiness, release buffers, or present frames.
    pub fn take_committed_frame_callbacks(&mut self) -> Vec<WlCallback> {
        let mut callbacks = Vec::new();
        for (_, state) in &mut self.cache {
            callbacks.append(&mut state.frame_callbacks);
        }
        callbacks
    }
}
