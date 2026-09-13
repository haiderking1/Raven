use super::{Placement, hints::Hints};
use crate::state::State;
use smithay::utils::IsAlive;
use std::sync::Mutex;

/// Window-owned history does not retain closed windows in a workspace map.
#[derive(Default)]
struct History {
    floating: Option<Placement>,
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
        let Some(area) = self.tiling_area() else {
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
            let floating = self.workspaces.entries[index]
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
                history.floating = floating;
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
                history.floating.take()
            };
            let placement = remembered.unwrap_or_else(|| {
                let size = window.geometry().size;
                let bounds = self.appearance.client_rect(area).size;
                Placement {
                    hints: Some(Hints::committed(&window)),
                    natural: Some(
                        (
                            size.w.min((bounds.w * 3 / 4).max(1)).max(1),
                            size.h.min((bounds.h * 3 / 4).max(1)).max(1),
                        )
                            .into(),
                    ),
                    ..Placement::default()
                }
            });
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
