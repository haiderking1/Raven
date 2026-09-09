use crate::state::State;
use smithay::{desktop::Window, utils::IsAlive};

impl State {
    pub(crate) fn map_layout_window(&mut self, index: usize, window: Window) {
        self.workspaces.entries[index]
            .floating
            .opening
            .remove(&window);
        if self.window_is_floating(&window) {
            self.commit_floating_size(&window);
            let origin = self.tiling_area().map(|a| a.loc).unwrap_or_default();
            self.workspaces.entries[index]
                .space
                .map_element(window, origin, false);
            self.arrange_floating(index);
        } else {
            self.map_tiled_window(index, window);
        }
    }

    pub(crate) fn forget_floating(&mut self, window: &Window) {
        if let Some(index) = self.workspaces.index_of(window) {
            let floating = &mut self.workspaces.entries[index].floating;
            floating.entries.remove(window);
            floating.opening.remove(window);
            floating.stack.retain(|w| w != window);
            floating.elevated.remove(window);
        }
    }

    pub(crate) fn transfer_floating(&mut self, window: &Window, source: usize, destination: usize) {
        if let Some(entry) = self.workspaces.entries[source]
            .floating
            .entries
            .remove(window)
        {
            self.workspaces.entries[destination]
                .floating
                .entries
                .insert(window.clone(), entry);
        }
        if let Some(hints) = self.workspaces.entries[source]
            .floating
            .opening
            .remove(window)
        {
            self.workspaces.entries[destination]
                .floating
                .opening
                .insert(window.clone(), hints);
        }
        self.workspaces.entries[source]
            .floating
            .stack
            .retain(|w| w != window);
        self.workspaces.entries[source]
            .floating
            .elevated
            .remove(window);
    }

    pub(crate) fn refresh_opening_floating(&mut self, index: usize) {
        let floating = &self.workspaces.entries[index].floating;
        let windows: Vec<_> = floating
            .opening
            .keys()
            .filter(|window| floating.entries.contains_key(*window))
            .filter(|window| {
                window
                    .toplevel()
                    .is_some_and(|top| top.is_initial_configure_sent())
            })
            .cloned()
            .collect();
        for window in windows {
            if self.fullscreen_manages(&window) {
                self.configure_fullscreen(&window, false);
            } else {
                self.configure_floating(&window);
                if let Some(top) = window.toplevel() {
                    top.send_pending_configure();
                }
            }
        }
    }

    pub(crate) fn refresh_floating(&mut self) {
        for index in 0..self.workspaces.entries.len() {
            let floating = &mut self.workspaces.entries[index].floating;
            let before = floating.entries.len();
            floating.entries.retain(|w, _| w.alive());
            floating.opening.retain(|w, _| w.alive());
            let workspace = &self.workspaces.entries[index];
            let stale = workspace.floating.stack.iter().any(|window| {
                !window.alive() || workspace.space.element_location(window).is_none()
            });
            if before != workspace.floating.entries.len() || stale {
                self.arrange_floating(index);
            }
        }
    }
}
