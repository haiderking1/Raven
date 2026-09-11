use smithay::wayland::compositor::{Blocker, BlockerState};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// This gate never owns or releases a GPU acquire fence.
pub(super) struct Coordination(pub Arc<AtomicBool>);

impl Blocker for Coordination {
    fn state(&self) -> BlockerState {
        if self.0.load(Ordering::Acquire) {
            BlockerState::Released
        } else {
            BlockerState::Pending
        }
    }
}
