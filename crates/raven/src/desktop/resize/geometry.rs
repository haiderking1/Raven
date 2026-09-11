use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle},
};

impl State {
    pub(crate) fn resize_displayed_frame(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        self.resize.held.get(window).map(|held| held.frame)
    }

    pub(crate) fn resize_displayed_client(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        self.resize.held.get(window).map(|held| held.client)
    }

    /// Layout writes target placement here; Space retains the displayed origin.
    /// Raising is deliberate: callers still own stacking and activation policy.
    pub(crate) fn place_resize_window(
        &mut self,
        index: usize,
        window: &Window,
        location: Point<i32, Logical>,
    ) {
        let displayed = if let Some(held) = self.resize.held.get_mut(window) {
            self.resize.dirty |= held.target_location != location;
            held.target_location = location;
            held.location
        } else {
            location
        };
        let space = &mut self.workspaces.entries[index].space;
        if space.element_location(window) != Some(displayed) {
            space.map_element(window.clone(), displayed, false);
        } else {
            space.raise_element(window, false);
        }
    }

    pub(super) fn update_resize_targets(&mut self) {
        let targets: Vec<_> = self
            .resize
            .held
            .keys()
            .map(|window| {
                (
                    window.clone(),
                    self.window_target_frame_geometry(window),
                    self.window_target_client_geometry(window),
                )
            })
            .collect();
        for (window, frame, client) in targets {
            let held = self.resize.held.get_mut(&window).unwrap();
            self.resize.dirty |= held.target_frame != frame || held.target_client != client;
            held.target_frame = frame;
            held.target_client = client;
        }
    }

    pub(super) fn publish_resize_windows(&mut self, windows: Vec<Window>) {
        if windows.is_empty() {
            return;
        }
        let mut changed = false;
        // map_element raises. Walk the existing stack, not hash-map order.
        for index in 0..self.workspaces.entries.len() {
            let stack: Vec<_> = self.workspaces.entries[index]
                .space
                .elements()
                .cloned()
                .collect();
            for window in stack {
                if windows.contains(&window) {
                    let Some(held) = self.resize.held.remove(&window) else {
                        continue;
                    };
                    // Fullscreen ownership and deferred grabs remain authoritative.
                    let location = self
                        .fullscreen_location(&window)
                        .or_else(|| {
                            self.window_target_client_geometry(&window)
                                .map(|area| area.loc)
                        })
                        .unwrap_or(held.target_location);
                    changed |= self.window_target_frame_geometry(&window) != Some(held.frame)
                        || self.window_target_client_geometry(&window) != Some(held.client)
                        || self.workspaces.entries[index]
                            .space
                            .element_location(&window)
                            != Some(location);
                    self.workspaces.entries[index]
                        .space
                        .map_element(window, location, false);
                } else {
                    self.workspaces.entries[index]
                        .space
                        .raise_element(&window, false);
                }
            }
        }
        // Unmapped/destroyed windows have no Space entry to visit.
        for window in windows {
            self.resize.held.remove(&window);
        }
        if !changed {
            return;
        }
        self.request_redraw();
        self.refresh_reactive_popups(self.workspaces.active);
        self.refresh_tiling_pointer();
    }
}
