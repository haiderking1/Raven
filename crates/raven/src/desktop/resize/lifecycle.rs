use crate::state::State;
use smithay::{desktop::Window, reexports::wayland_server::protocol::wl_surface::WlSurface};
use std::time::Instant;

impl State {
    /// Safe inside a pre-commit/destruction hook: defer queue re-entry to ping.
    pub(crate) fn cancel_resize_surface(&mut self, surface: &WlSurface) {
        self.cancel_resize_callbacks(Some(surface));
        let before = self.resize.held.len();
        self.resize.held.retain(|window, _| {
            !window
                .toplevel()
                .is_some_and(|top| top.wl_surface() == surface)
        });
        if before != self.resize.held.len()
            && let Some(batch) = &mut self.resize.batch
        {
            batch.deadline = Instant::now();
        }
        // A destroyed synchronized child can satisfy a recorded GPU wait.
        if self.resize.batch.is_some() {
            self.wake_resize_transactions();
        }
    }

    pub(crate) fn cancel_resize_window(&mut self, window: &Window) {
        if let Some(top) = window.toplevel() {
            self.cancel_resize_surface(top.wl_surface());
        }
    }

    /// Workspace/VT/output teardown discards layout holds, but never touches
    /// acquire blockers. Wake queued surface commits on the event source.
    pub(crate) fn cancel_resize_transactions(&mut self) {
        self.cancel_resize_callbacks(None);
        let windows = self.resize.held.keys().cloned().collect();
        self.publish_resize_windows(windows);
        if let Some(batch) = &mut self.resize.batch {
            batch.deadline = Instant::now();
        }
        self.wake_resize_transactions();
    }

    pub(crate) fn suspend_resize_transactions(&mut self) {
        self.resize.suspended = true;
        self.cancel_resize_transactions();
    }

    pub(crate) fn resume_resize_transactions(&mut self) {
        self.resize.suspended = false;
    }

    /// Called from desktop refresh before layout refresh. This is lifecycle
    /// validation, not a readiness poll; readiness is exclusively fd/timer driven.
    pub(crate) fn refresh_resize_lifecycle(&mut self) {
        let output = self.output.clone().zip(self.fullscreen_area());
        if self.resize.output != output {
            self.cancel_resize_transactions();
            self.resize.output = output;
        }
        if self
            .resize
            .batch
            .as_ref()
            .is_some_and(|batch| batch.workspace != self.workspaces.active)
        {
            self.cancel_resize_transactions();
        }
        let stale: Vec<_> = self
            .resize
            .held
            .keys()
            .filter(|window| {
                !self.window_is_visible(window)
                    || self.workspaces.index_of(window) != Some(self.workspaces.active)
            })
            .cloned()
            .collect();
        for window in stale {
            self.cancel_resize_window(&window);
        }
    }
}
