use crate::state::State;
use smithay::{
    reexports::wayland_server::{Resource, Weak, protocol::wl_surface::WlSurface},
    wayland::compositor::{get_children, is_sync_subsurface, with_states},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct SurfaceAcquire(Arc<AtomicUsize>);

/// A counter covers earlier dependent attachments as well as the newest one.
/// Dropping a cancelled readiness registration removes its contribution, but
/// does not release the separate Smithay GPU blocker.
pub(crate) struct Acquire(Arc<AtomicUsize>);

impl Acquire {
    pub(crate) fn complete(self, state: &State) {
        drop(self);
        state.wake_resize_transactions();
    }
}

impl Drop for Acquire {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) fn acquire_started(surface: &WlSurface) -> Acquire {
    with_states(surface, |states| {
        states.data_map.insert_if_missing(SurfaceAcquire::default);
        let count = states.data_map.get::<SurfaceAcquire>().unwrap().0.clone();
        count.fetch_add(1, Ordering::AcqRel);
        Acquire(count)
    })
}

pub(super) struct Readiness {
    count: Arc<AtomicUsize>,
    surface: Weak<WlSurface>,
}

impl Readiness {
    pub(super) fn is_ready(&self) -> bool {
        self.surface.upgrade().is_err() || self.count.load(Ordering::Acquire) == 0
    }
}

pub(super) fn tree(surface: &WlSurface) -> Vec<Readiness> {
    let mut waits = Vec::new();
    let mut remaining = vec![surface.clone()];
    while let Some(node) = remaining.pop() {
        // is_sync_subsurface locks the node and its ancestors. Smithay
        // traversal callbacks already hold those locks, so enumerate outside
        // with_states rather than querying synchronization inside a traversal.
        if &node != surface && !is_sync_subsurface(&node) {
            continue;
        }
        with_states(&node, |states| {
            if let Some(data) = states.data_map.get::<SurfaceAcquire>() {
                waits.push(Readiness {
                    count: data.0.clone(),
                    surface: node.downgrade(),
                });
            }
        });
        remaining.extend(
            get_children(&node)
                .into_iter()
                .filter(|child| child != &node),
        );
    }
    waits
}
