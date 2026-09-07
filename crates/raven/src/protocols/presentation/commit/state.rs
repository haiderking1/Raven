use smithay::{reexports::wayland_server::DisplayHandle, wayland::compositor::Cacheable};
use std::sync::Mutex;

#[derive(Clone, Copy)]
pub(super) struct Commit {
    pub serial: u64,
    pub has_feedback: bool,
}

#[derive(Default)]
pub(super) struct CommitState(pub Option<Commit>);

impl Cacheable for CommitState {
    fn commit(&mut self, _: &DisplayHandle) -> Self {
        Self(self.0.take())
    }
    fn merge_into(self, into: &mut Self, _: &DisplayHandle) {
        // Parent-driven synchronized merges without a fresh child commit must
        // not erase the child's identity, just as they must not apply it early.
        if self.0.is_some() {
            into.0 = self.0;
        }
    }
}

#[derive(Default)]
pub(super) struct Tracking(pub Mutex<Serials>);

#[derive(Default)]
pub(super) struct Serials {
    pub next: u64,
    pub applied: Option<u64>,
}
