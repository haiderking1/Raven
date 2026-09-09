use crate::state::State;
use smithay::{desktop::Window, wayland::shell::xdg::ToplevelSurface};

impl State {
    pub(crate) fn request_fullscreen(&mut self, surface: &ToplevelSurface, fullscreen: bool) {
        let Some(window) = self
            .windows
            .iter()
            .find(|w| w.toplevel() == Some(surface))
            .cloned()
        else {
            if surface.is_initial_configure_sent() {
                surface.send_configure();
            }
            return;
        };
        let Some(index) = self.workspaces.index_of(&window) else {
            return;
        };
        let workspace = &self.workspaces.entries[index];
        let mapped = workspace.space.element_location(&window).is_some();
        let eligible = !mapped
            || workspace.fullscreen.requested.as_ref() == Some(&window)
            || workspace.focused.as_ref() == Some(&window)
            || (index == self.workspaces.active && self.focused_window().as_ref() == Some(&window));
        if fullscreen && !eligible {
            // Refusal still replies, without replacing a legitimate in-flight mode.
            if surface.is_initial_configure_sent() {
                surface.send_configure();
            }
            return;
        }
        self.workspaces.entries[index]
            .fullscreen
            .entries
            .entry(window.clone())
            .or_default()
            .intent = fullscreen;
        if mapped && fullscreen {
            self.claim_fullscreen(index, &window);
        } else if !fullscreen
            && self.workspaces.entries[index].fullscreen.requested.as_ref() == Some(&window)
        {
            self.workspaces.entries[index].fullscreen.requested = None;
        }
        self.configure_fullscreen(&window, true);
    }

    pub(crate) fn toggle_fullscreen(&mut self) {
        let Some(window) = self.focused_window() else {
            return;
        };
        let Some(index) = self.workspaces.index_of(&window) else {
            return;
        };
        let full = self.workspaces.entries[index]
            .fullscreen
            .entries
            .get(&window)
            .is_some_and(|e| e.intent);
        if let Some(surface) = window.toplevel() {
            self.request_fullscreen(surface, !full);
        }
    }

    pub(crate) fn claim_fullscreen(&mut self, index: usize, window: &Window) {
        let fullscreen = &mut self.workspaces.entries[index].fullscreen;
        let previous = fullscreen.requested.replace(window.clone());
        if let Some(previous) = previous.filter(|previous| previous != window) {
            if let Some(entry) = fullscreen.entries.get_mut(&previous) {
                entry.intent = false;
            }
            // Invalidate an enter even if it has not displayed yet. A stale ACK
            // from a displaced claimant must never resurrect its ownership.
            self.configure_fullscreen(&previous, true);
        }
    }

    pub(crate) fn map_fullscreen_intent(&mut self, index: usize, window: &Window) {
        if self.workspaces.entries[index]
            .fullscreen
            .entries
            .get(window)
            .is_some_and(|e| e.intent)
        {
            self.claim_fullscreen(index, window);
            self.configure_fullscreen(window, false);
        }
    }

    pub(crate) fn clear_fullscreen(&mut self, window: &Window) {
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let fullscreen = &mut self.workspaces.entries[index].fullscreen;
        fullscreen.entries.remove(window);
        if fullscreen.requested.as_ref() == Some(window) {
            fullscreen.requested = None;
        }
        if fullscreen.displayed.as_ref() == Some(window) {
            fullscreen.displayed = None;
        }
    }
}
