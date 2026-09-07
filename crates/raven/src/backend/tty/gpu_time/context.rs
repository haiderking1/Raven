//! A lease prevents two collectors from consuming one context's disjoint flag.

use smithay::backend::renderer::gles::GlesRenderer;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Owners(Arc<Mutex<HashSet<usize>>>);

pub(super) struct Lease {
    owners: Arc<Mutex<HashSet<usize>>>,
    handle: usize,
}

impl Lease {
    pub(super) fn acquire(renderer: &GlesRenderer) -> Option<Self> {
        let context = renderer.egl_context();
        let owners = &context
            .user_data()
            .get_or_insert_threadsafe(Owners::default)
            .0;
        let handle = context.get_context_handle() as usize;
        if !owners
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(handle)
        {
            return None;
        }
        Some(Self {
            owners: owners.clone(),
            handle,
        })
    }

    pub(super) fn matches(&self, renderer: &GlesRenderer) -> bool {
        let context = renderer.egl_context();
        context.get_context_handle() as usize == self.handle
            && context
                .user_data()
                .get::<Owners>()
                .is_some_and(|owners| Arc::ptr_eq(&self.owners, &owners.0))
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.owners
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.handle);
    }
}
