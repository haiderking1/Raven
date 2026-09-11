use crate::state::{ClientState, State};
use std::sync::atomic::Ordering;

impl State {
    /// Commit callbacks can reveal windows, change floating hints or remove
    /// members. Recompute after publishing this cohort, not halfway through
    /// applying it. The fixed workspace bitmap cannot grow with client input.
    pub(crate) fn defer_resize_layout(&mut self, index: usize) -> bool {
        if !self.resize.releasing {
            return false;
        }
        self.resize.deferred_layout[index] = true;
        true
    }

    pub(crate) fn resize_is_applying(&self) -> bool {
        self.resize.releasing
    }

    pub(super) fn release_resize_transaction(&mut self) {
        if self.resize.releasing {
            return;
        }
        let Some(batch) = self.resize.batch.take() else {
            return;
        };
        if let Some(token) = self.resize.timer.take()
            && let Some(handle) = &self.resize.handle
        {
            handle.remove(token);
        }
        self.resize.releasing = true;
        batch.released.store(true, Ordering::Release);
        // Keep displayed allocations pinned throughout all reentrant commit
        // callbacks. Only then publish the windows whose state really applied.
        for client in batch.clients {
            if let Some(data) = client.get_data::<ClientState>() {
                let display = self.display_handle.clone();
                data.compositor_state.blocker_cleared(self, &display);
            }
        }
        self.resize.releasing = false;
        let applied = self
            .resize
            .held
            .iter()
            .filter_map(|(window, held)| {
                (held.configure.is_none() || held.applied).then_some(window.clone())
            })
            .collect();
        self.publish_resize_windows(applied);
        let deferred = std::mem::take(&mut self.resize.deferred_layout);
        for (index, pending) in deferred.into_iter().enumerate() {
            if !pending {
                continue;
            }
            self.begin_resize_batch(index);
            self.refresh_workspace_tiling(index);
            self.retile_workspace(index);
            self.position_fullscreen_windows(index);
            self.end_resize_batch();
        }
        // A timed-out client's old displayed allocation remains until its real
        // matching commit applies. It no longer holds up any other participant.
    }
}
