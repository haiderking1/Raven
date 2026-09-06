use super::{configure::configure_tile, geometry::master_stack};
use crate::state::State;
use smithay::{
    input::pointer::MotionEvent,
    utils::{Clock, Logical, Monotonic, Rectangle, SERIAL_COUNTER},
};

impl State {
    pub(super) fn tiling_area(&self) -> Option<Rectangle<i32, Logical>> {
        self.output
            .as_ref()
            .and_then(|output| self.space().output_geometry(output))
    }

    /// Membership and geometry belong to a workspace, not the visible output.
    pub(crate) fn retile_workspace(&mut self, index: usize) {
        let area = self.tiling_area();
        let workspace = &mut self.workspaces.entries[index];
        workspace.tiling.geometry = area;
        if let Some(area) = area {
            let tiles = master_stack(area, workspace.tiling.windows.len());
            // map_element raises even with activate=false. Preserve stacking.
            let stacking: Vec<_> = workspace.space.elements().cloned().collect();
            for window in stacking {
                let Some(index) = workspace
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
                if workspace.space.element_location(&window) != Some(tile.loc) {
                    workspace.space.map_element(window, tile.loc, false);
                } else {
                    workspace.space.raise_element(&window, false);
                }
            }
        }
        if index == self.workspaces.active {
            self.refresh_tiling_pointer();
        }
    }

    pub(crate) fn refresh_tiling_pointer(&mut self) {
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
