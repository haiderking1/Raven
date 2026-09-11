use crate::state::{ClientState, State};
use smithay::{
    desktop::Window,
    reexports::wayland_server::{Client, Resource},
    utils::Serial,
    wayland::compositor::{Blocker, BlockerState},
};
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Default)]
pub(super) struct FullscreenCommits {
    pending: VecDeque<Serial>,
    waiting: Option<Arc<AtomicBool>>,
}

pub(super) struct FullscreenCoordination {
    pub(super) batch: Arc<AtomicBool>,
    pub(super) response: Arc<AtomicBool>,
}

impl Blocker for FullscreenCoordination {
    fn state(&self) -> BlockerState {
        if self.batch.load(Ordering::Acquire) || self.response.load(Ordering::Acquire) {
            BlockerState::Released
        } else {
            BlockerState::Pending
        }
    }
}

impl State {
    pub(crate) fn track_fullscreen_resize_configure(&mut self, window: &Window, serial: Serial) {
        let previous = self.resize.held.get(window).and_then(|held| held.configure);
        self.track_resize_configure(window, Some(serial));
        let Some(held) = self.resize.held.get_mut(window) else {
            return;
        };
        if held.configure == previous {
            return;
        }
        held.fullscreen_commits
            .get_or_insert_with(Default::default)
            .pending
            .push_back(serial);
    }

    // Consume only transactions covered by this commit's ACK. Older content and
    // ordinary commits have no new transaction and must not acquire a new gate.
    pub(super) fn fullscreen_resize_response(
        &mut self,
        window: &Window,
        serial: Serial,
    ) -> Option<Arc<AtomicBool>> {
        let commits = self
            .resize
            .held
            .get_mut(window)?
            .fullscreen_commits
            .as_mut()?;
        let mut consumed = false;
        while commits
            .pending
            .front()
            .is_some_and(|pending| serial >= *pending)
        {
            commits.pending.pop_front();
            consumed = true;
        }
        if !consumed {
            return None;
        }
        if let Some(previous) = commits.waiting.take() {
            previous.store(true, Ordering::Release);
            if let Some(client) = window.toplevel().and_then(|top| top.wl_surface().client()) {
                if !self.resize.fullscreen_notifications.contains(&client) {
                    self.resize.fullscreen_notifications.push(client);
                }
            }
        }
        // A reply to a superseded configure completes that request. It cannot be
        // made to wait for the newer configure that this client has yet to draw.
        if !commits.pending.is_empty() {
            return None;
        }
        let response = Arc::new(AtomicBool::new(false));
        commits.waiting = Some(response.clone());
        Some(response)
    }

    pub(super) fn notify_fullscreen_resize_clients(&mut self) {
        let clients: Vec<Client> = std::mem::take(&mut self.resize.fullscreen_notifications);
        for client in clients {
            if let Some(data) = client.get_data::<ClientState>() {
                let display = self.display_handle.clone();
                data.compositor_state.blocker_cleared(self, &display);
            }
        }
    }
}
