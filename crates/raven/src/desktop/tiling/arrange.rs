use super::{configure::configure_tile, geometry::adjusted};
use crate::state::State;
use smithay::utils::{Clock, Logical, Monotonic, Rectangle, SERIAL_COUNTER};

impl State {
    pub(crate) fn tiling_area(&self) -> Option<Rectangle<i32, Logical>> {
        let output = self.output.as_ref()?;
        let area = self.space().output_geometry(output)?;
        if area.size.w <= 0 || area.size.h <= 0 {
            return None;
        }
        // Drop the guard before any window or floating-parent allocation lookup.
        let zone = {
            let map = smithay::desktop::layer_map_for_output(output);
            if map.len() == 0 {
                area
            } else {
                let mut zone = map.non_exclusive_zone();
                zone.loc += area.loc;
                zone
            }
        };
        let zone = zone
            .intersection(area)
            .filter(|z| z.size.w > 0 && z.size.h > 0)
            .unwrap_or_else(|| {
                Rectangle::new(
                    (
                        zone.loc.x.clamp(area.loc.x, area.loc.x + area.size.w - 1),
                        zone.loc.y.clamp(area.loc.y, area.loc.y + area.size.h - 1),
                    )
                        .into(),
                    (1, 1).into(),
                )
            });
        Some(self.appearance.inset_workarea(zone))
    }

    /// Membership and geometry belong to a workspace, not the visible output.
    pub(crate) fn retile_workspace(&mut self, index: usize) {
        if self.defer_resize_layout(index) {
            return;
        }
        self.begin_resize_batch(index);
        let area = self.tiling_area();
        self.workspaces.entries[index].tiling.geometry = area;
        if let Some(area) = area {
            let workspace = &mut self.workspaces.entries[index];
            // Row proportions describe slots in the current stack. A membership
            // count change starts a new equal stack rather than reviving stale weights.
            if workspace.tiling.splits.rows.len()
                != workspace.tiling.windows.len().saturating_sub(1)
            {
                workspace.tiling.splits.rows.clear();
            }
            workspace.tiling.frames = adjusted::frames(
                area,
                workspace.tiling.windows.len(),
                self.appearance.inner,
                &workspace.tiling.splits,
            );
            let tiles = workspace.tiling.frames.clone();
            // map_element raises even with activate=false. Preserve stacking.
            let stacking: Vec<_> = workspace.space.elements().cloned().collect();
            let members = workspace.tiling.windows.clone();
            for window in stacking {
                let Some(position) = members.iter().position(|candidate| candidate == &window)
                else {
                    continue;
                };
                let Some(tile) = tiles.get(position) else {
                    continue;
                };
                let tile = self.appearance.client_rect(*tile);
                if self.fullscreen_manages(&window) {
                    // Includes displaced claimants with an uncommitted exit.
                    self.configure_fullscreen(&window, false);
                } else {
                    if !self.configure_transient(&window) {
                        configure_tile(&window, tile);
                    }
                    self.send_resize_configure(&window);
                }
                let location = self
                    .fullscreen_location(&window)
                    .or_else(|| {
                        self.fullscreen_transient_geometry(&window)
                            .map(|area| area.loc)
                    })
                    .unwrap_or(tile.loc);
                self.place_resize_window(index, &window, location);
            }
        }
        self.arrange_floating(index);
        self.refresh_opening_floating(index);
        self.refresh_reactive_popups(index);
        self.end_resize_batch();
        if index == self.workspaces.active {
            self.request_redraw();
            self.refresh_tiling_pointer();
        }
    }

    pub(crate) fn refresh_tiling_pointer(&mut self) {
        if self.resize_is_applying() || self.screenshot.active() {
            return;
        }
        let time = Clock::<Monotonic>::new().now().as_millis();
        if self.refresh_pointer_focus(SERIAL_COUNTER.next_serial(), time)
            && let Some(pointer) = self.seat.get_pointer()
        {
            pointer.frame(self);
        }
    }
}
