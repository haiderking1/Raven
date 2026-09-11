use crate::state::State;

impl State {
    pub(crate) fn position_fullscreen_windows(&mut self, index: usize) {
        if self.defer_resize_layout(index) {
            return;
        }
        if self.workspaces.entries[index]
            .fullscreen
            .displayed
            .is_none()
        {
            return;
        }
        self.begin_resize_batch(index);
        let windows: Vec<_> = self.workspaces.entries[index]
            .space
            .elements()
            .cloned()
            .collect();
        let positions: Vec<_> = windows
            .iter()
            .map(|w| {
                if !self.fullscreen_manages(w)
                    && self.configure_transient(w)
                    && let Some(toplevel) = w.toplevel()
                    && toplevel.is_initial_configure_sent()
                {
                    self.send_resize_configure(w);
                }
                self.fullscreen_location(w)
                    .or_else(|| self.fullscreen_transient_geometry(w).map(|area| area.loc))
            })
            .collect();
        let space = &mut self.workspaces.entries[index].space;
        let changed = windows
            .iter()
            .zip(&positions)
            .any(|(w, p)| p.is_some() && space.element_location(w) != *p);
        if !changed {
            self.end_resize_batch();
            return;
        }
        // Mapping raises. Rebuild the existing stack order, never the tile order.
        for (window, position) in windows.into_iter().zip(positions) {
            if let Some(position) = position {
                self.place_resize_window(index, &window, position);
            } else {
                self.workspaces.entries[index]
                    .space
                    .raise_element(&window, false);
            }
        }
        self.arrange_floating(index);
        self.refresh_reactive_popups(index);
        self.end_resize_batch();
        if index == self.workspaces.active {
            self.request_redraw();
            self.refresh_tiling_pointer();
        }
    }
}
