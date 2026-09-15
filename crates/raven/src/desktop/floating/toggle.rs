mod geometry;
use crate::state::State;
use smithay::utils::{IsAlive, Logical, Size};
use std::sync::Mutex;

/// Window-owned history does not retain closed windows in a workspace map.
#[derive(Default)]
struct History {
    floating_size: Option<Size<i32, Logical>>,
    tile: Option<usize>,
}

impl State {
    pub(crate) fn toggle_focused_floating(&mut self) {
        let Some(window) = self.focused_window() else {
            return;
        };
        let Some(index) = self.workspaces.index_of(&window) else {
            return;
        };
        if index != self.workspaces.active
            || !window.alive()
            || window.toplevel().is_none()
            || !self.window_is_visible(&window)
            || self.space().element_location(&window).is_none()
            || self.fullscreen_manages(&window)
            || self.pointer_is_captured()
            || self
                .seat
                .get_keyboard()
                .is_some_and(|keyboard| keyboard.is_grabbed())
            || self
                .seat
                .get_pointer()
                .is_some_and(|pointer| pointer.is_grabbed())
        {
            return;
        }
        // Leave the explicit maximized mode before another placement change.
        if super::maximize::is_maximized(&window) {
            self.set_window_maximized(&window, false);
            return;
        }
        let Some(frame) = self.window_frame_geometry(&window) else {
            return;
        };
        let Some(client) = self.window_client_geometry(&window) else {
            return;
        };
        let was_floating = self.window_is_floating(&window);
        if !was_floating && !self.window_is_tiled(&window) {
            return;
        }
        window
            .user_data()
            .insert_if_missing(|| Mutex::new(History::default()));
        self.end_live_resize(&window);
        // Capture old allocations before changing either membership list.
        self.begin_resize_batch(index);
        if was_floating {
            self.workspaces.entries[index]
                .floating
                .entries
                .remove(&window);
            let slot = {
                let mut history = window
                    .user_data()
                    .get::<Mutex<History>>()
                    .unwrap()
                    .lock()
                    .unwrap();
                history.floating_size = Some(client.size);
                history.tile
            };
            self.restore_floating_tile(&window, slot);
        } else {
            let slot = self.detach_floating_tile(&window);
            let remembered = {
                let mut history = window
                    .user_data()
                    .get::<Mutex<History>>()
                    .unwrap()
                    .lock()
                    .unwrap();
                history.tile = slot;
                history.floating_size
            };
            let placement = geometry::placement(&window, frame, client, remembered);
            self.workspaces.entries[index]
                .floating
                .entries
                .insert(window.clone(), placement);
        }
        self.retile_workspace(index);
        self.end_resize_batch();
        self.activate_window(Some(window));
        self.request_redraw();
    }
}
