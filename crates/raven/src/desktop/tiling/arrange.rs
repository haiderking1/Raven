use super::{configure::configure_tile, geometry::master_stack};
use crate::state::State;
use smithay::{
    desktop::Window,
    input::pointer::MotionEvent,
    utils::{Clock, IsAlive, Logical, Monotonic, Rectangle, SERIAL_COUNTER},
};

impl State {
    fn tiling_area(&self) -> Option<Rectangle<i32, Logical>> {
        self.output
            .as_ref()
            .and_then(|output| self.space.output_geometry(output))
    }

    /// Size the first configure without reserving space for an unmapped client.
    pub(crate) fn configure_initial_tile(&self, window: &Window) {
        if let Some(area) = self.tiling_area()
            && let Some(tile) = master_stack(area, self.tiling.windows.len() + 1).last()
        {
            configure_tile(window, *tile);
        }
    }

    pub(crate) fn map_tiled_window(&mut self, window: Window) {
        if !self.tiling.windows.contains(&window) {
            self.tiling.windows.push(window.clone());
        }
        let origin = self.tiling_area().map(|area| area.loc).unwrap_or_default();
        self.space.map_element(window, origin, false);
        self.retile_windows();
    }

    /// Called after membership changes or output geometry changes, not every frame.
    pub(crate) fn retile_windows(&mut self) {
        self.tiling.geometry = self.tiling_area();
        let Some(area) = self.tiling.geometry else {
            return;
        };
        let tiles = master_stack(area, self.tiling.windows.len());
        // map_element raises its argument even with activate=false. Visit windows
        // in the old stacking order so a resize does not change focus or stacking.
        let stacking: Vec<_> = self.space.elements().cloned().collect();
        for window in stacking {
            let Some(index) = self
                .tiling
                .windows
                .iter()
                .position(|candidate| candidate == &window)
            else {
                continue;
            };
            let Some(tile) = tiles.get(index) else {
                continue;
            };
            configure_tile(&window, *tile);
            if let Some(toplevel) = window.toplevel() {
                toplevel.send_pending_configure();
            }
            if self.space.element_location(&window) != Some(tile.loc) {
                self.space.map_element(window, tile.loc, false);
            } else {
                // Moving a different window can have raised it above this one.
                self.space.raise_element(&window, false);
            }
        }
        self.refresh_tiling_pointer();
    }

    pub(crate) fn refresh_tiling(&mut self) {
        let before = self.tiling.windows.len();
        self.tiling
            .windows
            .retain(|window| window.alive() && self.space.element_location(window).is_some());
        if before != self.tiling.windows.len() || self.tiling.geometry != self.tiling_area() {
            self.retile_windows();
        }
    }

    fn refresh_tiling_pointer(&mut self) {
        if let Some(pointer) = self.seat.get_pointer() {
            let focus = self.surface_under(self.pointer_location);
            pointer.motion(
                self,
                focus,
                &MotionEvent {
                    location: self.pointer_location,
                    serial: SERIAL_COUNTER.next_serial(),
                    time: Clock::<Monotonic>::new().now().as_millis(),
                },
            );
            pointer.frame(self);
        }
    }
}
