use super::Placement;
use crate::state::State;
use smithay::{desktop::Window, utils::IsAlive};
use std::sync::Mutex;
#[derive(Default)]
struct Maximized {
    saved: Option<Saved>,
}
enum Saved {
    Floating(Placement),
    Tiled(Option<usize>),
    Automatic,
}
pub(crate) fn is_maximized(window: &Window) -> bool {
    window
        .user_data()
        .get::<Mutex<Maximized>>()
        .is_some_and(|m| m.lock().unwrap().saved.is_some())
}
pub(super) fn resolve_opening(window: &Window, floating: bool) {
    if let Some(m) = window.user_data().get::<Mutex<Maximized>>() {
        let mut m = m.lock().unwrap();
        if matches!(m.saved, Some(Saved::Automatic)) {
            m.saved = Some(if floating {
                Saved::Floating(Placement::default())
            } else {
                Saved::Tiled(None)
            });
        }
    }
}
pub(crate) fn clear(window: &Window) {
    if let Some(m) = window.user_data().get::<Mutex<Maximized>>() {
        m.lock().unwrap().saved = None;
    }
}
impl State {
    pub(crate) fn set_window_maximized(&mut self, window: &Window, maximized: bool) -> bool {
        let Some(index) = self.workspaces.index_of(window) else {
            return false;
        };
        if !window.alive()
            || window.toplevel().is_none()
            || self.fullscreen_manages(window)
            || self.pointer_is_captured()
        {
            return false;
        }
        if is_maximized(window) == maximized {
            return true;
        }
        let minimized = self.window_is_minimized(window);
        window
            .user_data()
            .insert_if_missing(|| Mutex::new(Maximized::default()));
        let mapped = self.workspaces.entries[index]
            .space
            .element_location(window)
            .is_some();
        self.end_live_resize(window);
        self.begin_resize_batch(index);
        if maximized {
            let saved = if let Some(placement) =
                self.workspaces.entries[index].floating.entries.get(window)
            {
                Saved::Floating(placement.clone())
            } else if mapped {
                Saved::Tiled(
                    self.detach_floating_tile(window)
                        .or_else(|| crate::desktop::management::minimize::tile(window)),
                )
            } else {
                Saved::Automatic
            };
            window
                .user_data()
                .get::<Mutex<Maximized>>()
                .unwrap()
                .lock()
                .unwrap()
                .saved = Some(saved);
            self.workspaces.entries[index]
                .floating
                .entries
                .entry(window.clone())
                .or_default();
        } else {
            let saved = window
                .user_data()
                .get::<Mutex<Maximized>>()
                .unwrap()
                .lock()
                .unwrap()
                .saved
                .take()
                .unwrap();
            match saved {
                Saved::Floating(placement) => {
                    self.workspaces.entries[index]
                        .floating
                        .entries
                        .insert(window.clone(), placement);
                }
                Saved::Tiled(slot) => {
                    self.workspaces.entries[index]
                        .floating
                        .entries
                        .remove(window);
                    if mapped && !minimized {
                        self.restore_floating_tile(window, slot);
                    } else if minimized {
                        crate::desktop::management::minimize::remember_tile(window, slot);
                    }
                }
                Saved::Automatic => {
                    self.workspaces.entries[index]
                        .floating
                        .entries
                        .remove(window);
                    if mapped {
                        self.restore_floating_tile(window, None);
                    }
                }
            }
        }
        self.retile_workspace(index);
        self.end_resize_batch();
        self.request_redraw();
        true
    }
}

impl State {
    /// An explicit manual move/resize replaces the maximized allocation with the
    /// currently displayed floating geometry; it must not snap back next frame.
    pub(crate) fn leave_maximized_for_interaction(&mut self, window: &Window) {
        if !is_maximized(window) {
            return;
        }
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let (Some(frame), Some(client)) = (
            self.window_frame_geometry(window),
            self.window_client_geometry(window),
        ) else {
            return;
        };
        clear(window);
        if let Some(entry) = self.workspaces.entries[index]
            .floating
            .entries
            .get_mut(window)
        {
            entry.position = Some(frame.loc);
            entry.natural = Some(client.size);
            entry.manual_size = true;
            entry.frame = Some(frame);
            entry.geometry = Some(client);
        }
        self.configure_floating(window);
        if let Some(top) = window.toplevel() {
            top.send_pending_configure();
        }
    }
}
