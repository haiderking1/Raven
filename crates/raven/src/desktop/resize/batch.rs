use super::{Batch, Configure, Held};
use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::Serial,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

impl State {
    /// Pair even on hidden workspaces. Nested arrange/configure operations share
    /// the outer cohort, and no event dispatch occurs while it is being built.
    pub(crate) fn begin_resize_batch(&mut self, index: usize) {
        self.resize.depth += 1;
        if self.resize.depth != 1 {
            return;
        }
        self.resize.dirty = false;
        self.resize.collecting = !self.resize.releasing
            && !self.resize.suspended
            && index == self.workspaces.active
            && self.fullscreen_area().is_some();
        if !self.resize.collecting {
            return;
        }
        if self.resize.output.is_none() {
            self.resize.output = self.output.clone().zip(self.fullscreen_area());
        }
        self.resize.fresh = self.resize.batch.is_none();
        if self.resize.fresh {
            self.resize.batch = Some(Batch {
                workspace: index,
                released: Arc::new(AtomicBool::new(false)),
                deadline: Instant::now() + Duration::from_millis(300),
                clients: Vec::new(),
            });
        }
        let windows: Vec<_> = self.workspaces.entries[index]
            .space
            .elements()
            .filter(|w| w.toplevel().is_some() && self.window_is_visible(w))
            .cloned()
            .collect();
        for window in windows {
            use smithay::reexports::wayland_server::Resource;
            if let Some(client) = window.toplevel().and_then(|top| top.wl_surface().client()) {
                let clients = &mut self.resize.batch.as_mut().unwrap().clients;
                if !clients.contains(&client) {
                    clients.push(client);
                }
            }
            if self.resize.held.contains_key(&window) {
                continue;
            }
            let Some(frame) = self.window_frame_geometry(&window) else {
                continue;
            };
            let Some(client) = self.window_client_geometry(&window) else {
                continue;
            };
            let Some(location) = self.workspaces.entries[index]
                .space
                .element_location(&window)
            else {
                continue;
            };
            self.resize.held.insert(
                window,
                Held {
                    frame,
                    client,
                    location,
                    target_location: location,
                    target_frame: Some(frame),
                    target_client: Some(client),
                    configure: None,
                    ready: None,
                    applied: false,
                    queued: false,
                },
            );
        }
    }

    pub(crate) fn end_resize_batch(&mut self) {
        assert!(self.resize.depth > 0, "unpaired resize batch");
        self.resize.depth -= 1;
        if self.resize.depth != 0 || self.resize.releasing || !self.resize.collecting {
            return;
        }
        if self.resize.batch.is_none() {
            return;
        }
        self.update_resize_targets();
        if self.resize.fresh && !self.resize.dirty {
            // No request or target changed, and dispatch cannot run inside
            // collection. Nothing references this candidate gate. In particular
            // do not poll client queues or re-arm a timed-out window here.
            self.resize.batch = None;
            let unchanged = self
                .resize
                .held
                .iter()
                .filter_map(|(window, held)| {
                    (held.configure.is_none() || held.applied).then_some(window.clone())
                })
                .collect();
            self.publish_resize_windows(unchanged);
            return;
        }
        if self
            .resize
            .held
            .values()
            .all(|held| held.configure.is_none() || held.applied)
        {
            self.release_resize_transaction();
        } else {
            self.arm_resize_deadline();
            if self.resize.dirty {
                self.wake_resize_transactions();
            }
        }
    }

    /// Record the serial that actually went over the wire. A no-op pending
    /// configure preserves the previous expectation; newer mode/size requests
    /// replace it, rather than retaining one transaction for every input event.
    pub(crate) fn track_resize_configure(&mut self, window: &Window, serial: Option<Serial>) {
        let Some(serial) = serial else {
            return;
        };
        if self.resize.releasing {
            return;
        }
        let Some(top) = window.toplevel() else {
            return;
        };
        let configure = with_states(top.wl_surface(), |states| {
            let data = states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap();
            let target = data.current_server_state();
            Configure {
                serial,
                size: target.size,
                fullscreen: target.states.contains(xdg_toplevel::State::Fullscreen),
            }
        });
        if let Some(held) = self.resize.held.get_mut(window) {
            // Keep readiness across a newer activation/bounds-only configure
            // carrying exactly the same resize target. Its earlier reply suffices.
            if held.configure.is_some_and(|old| {
                old.size == configure.size && old.fullscreen == configure.fullscreen
            }) {
                return;
            }
            self.resize.dirty = true;
            held.queued = false;
            held.configure = Some(configure);
            held.ready = None;
            held.applied = false;
        }
    }
}
