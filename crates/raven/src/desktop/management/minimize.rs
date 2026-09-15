use super::model::{Minimized, with};
pub(crate) use super::model::{clear, hidden, remember_tile, tile};
use crate::state::State;
use smithay::desktop::Window;
use std::sync::Mutex;
impl State {
    pub(crate) fn set_window_minimized(&mut self, window: &Window, minimized: bool) -> bool {
        let Some(index) = self.workspaces.index_of(window) else {
            return false;
        };
        if self.workspaces.entries[index]
            .space
            .element_location(window)
            .is_none()
            || self.pointer_is_captured()
        {
            return false;
        }
        // Never hide the last restore route. A compatible taskbar/Dock supplies it.
        if minimized
            && (self.is_configuration_error(window) || !self.foreign_toplevel.can_restore(window))
        {
            return false;
        }
        with(window, |v| v.requested = minimized);
        if minimized && self.fullscreen_manages(window) {
            if let Some(top) = window.toplevel() {
                self.request_fullscreen(top, false);
            }
        }
        self.refresh_minimized_windows();
        true
    }
    pub(crate) fn refresh_minimized_windows(&mut self) {
        if !self.windows.iter().any(|window| {
            window
                .user_data()
                .get::<Mutex<Minimized>>()
                .is_some_and(|state| {
                    let state = state.lock().unwrap();
                    state.requested || state.hidden
                })
        }) {
            return;
        }
        let windows = self.windows.clone();
        for window in &windows {
            if self.is_configuration_error(window) || !self.foreign_toplevel.can_restore(window) {
                if let Some(v) = window.user_data().get::<Mutex<Minimized>>() {
                    v.lock().unwrap().requested = false;
                }
            }
        }
        let mut dirty = std::collections::BTreeSet::new();
        for window in windows {
            let Some(index) = self.workspaces.index_of(&window) else {
                continue;
            };
            if self.workspaces.entries[index]
                .space
                .element_location(&window)
                .is_none()
            {
                continue;
            }
            let wanted = self.wants_minimized(&window);
            if wanted == hidden(&window) || (wanted && self.fullscreen_manages(&window)) {
                continue;
            }
            if wanted {
                self.cancel_resize_window(&window);
            }
            self.begin_resize_batch(index);
            if wanted {
                if let Some(top) = window.toplevel() {
                    self.dismiss_window_popups(top.wl_surface());
                }
                let tile = self.detach_floating_tile(&window).or_else(|| tile(&window));
                with(&window, |v| {
                    v.hidden = true;
                    v.tile = tile;
                });
            } else {
                let tile = with(&window, |v| {
                    v.hidden = false;
                    v.tile.take()
                });
                if !self.window_is_floating(&window) {
                    self.restore_floating_tile(&window, tile);
                }
            }
            self.retile_workspace(index);
            self.end_resize_batch();
            dirty.insert(index);
        }
        if !dirty.is_empty() {
            self.restore_focus();
            self.refresh_tiling_pointer();
            self.request_redraw();
        }
    }
}
