use crate::state::State;
use smithay::{
    backend::allocator::dmabuf::{Dmabuf, DmabufSource},
    reexports::{
        calloop::{LoopHandle, RegistrationToken},
        wayland_server::backend::ObjectId,
    },
};
use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

struct Wait {
    token: RegistrationToken,
    surface: ObjectId,
    cancelled: Arc<AtomicBool>,
}

/// Sources remove themselves when ready. Keep only live tokens so destroying
/// a surface or stopping the backend also closes never-signalled fence fds.
pub(super) struct Pending {
    handle: LoopHandle<'static, State>,
    waits: Rc<RefCell<HashMap<u64, Wait>>>,
    next_id: u64,
}

impl Pending {
    pub fn new(handle: LoopHandle<'static, State>) -> Self {
        Self {
            handle,
            waits: Rc::default(),
            next_id: 0,
        }
    }

    pub fn insert(
        &mut self,
        source: DmabufSource,
        surface: ObjectId,
        cancelled: Arc<AtomicBool>,
        mut callback: impl FnMut(&mut Dmabuf, &mut State) -> io::Result<()> + 'static,
    ) -> io::Result<()> {
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or_else(|| io::Error::other("DMA-BUF readiness ids exhausted"))?;
        let waits = self.waits.clone();
        let token = self
            .handle
            .insert_source(source, move |(), dmabuf, state| {
                waits.borrow_mut().remove(&id);
                // Release the map borrow before blocker_cleared re-enters State.
                callback(dmabuf, state)
            })
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.waits.borrow_mut().insert(
            id,
            Wait {
                token,
                surface,
                cancelled,
            },
        );
        Ok(())
    }

    pub fn remove_surface(&mut self, surface: &ObjectId) {
        let ids: Vec<_> = self
            .waits
            .borrow()
            .iter()
            .filter_map(|(id, wait)| (&wait.surface == surface).then_some(*id))
            .collect();
        for id in ids {
            let wait = self.waits.borrow_mut().remove(&id);
            if let Some(wait) = wait {
                wait.cancelled.store(true, Ordering::SeqCst);
                self.handle.remove(wait.token);
            }
        }
    }

    pub fn clear(&mut self) {
        let waits = std::mem::take(&mut *self.waits.borrow_mut());
        for (_, wait) in waits {
            wait.cancelled.store(true, Ordering::SeqCst);
            self.handle.remove(wait.token);
        }
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        self.clear();
    }
}
