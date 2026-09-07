use super::{configure::configure_tile, geometry::master_stack};
use crate::state::State;
use smithay::utils::{Clock, Logical, Monotonic, Rectangle, SERIAL_COUNTER};

impl State {
    pub(super) fn tiling_area(&self) -> Option<Rectangle<i32, Logical>> {
        let output = self.output.as_ref()?;
        let area = self.space().output_geometry(output)?;
        let map = smithay::desktop::layer_map_for_output(output);
        if map.len() == 0 {
            return Some(area);
        }
        let mut zone = map.non_exclusive_zone();
        zone.loc += area.loc;
        Some(zone)
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
            self.request_redraw();
            self.refresh_tiling_pointer();
        }
    }

    pub(crate) fn refresh_tiling_pointer(&mut self) {
        let time = Clock::<Monotonic>::new().now().as_millis();
        if self.refresh_pointer_focus(SERIAL_COUNTER.next_serial(), time)
            && let Some(pointer) = self.seat.get_pointer()
        {
            pointer.frame(self);
        }
    }
}
