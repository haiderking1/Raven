use super::cache::{Animations, PENDING_LIFETIME};
use crate::{
    desktop::animation::{geometry::Geometry, timeline::Timeline},
    state::State,
};
use smithay::reexports::wayland_server::Resource;
use smithay::{
    output::Output,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};
use std::time::Instant;

impl Animations {
    /// Called after desktop reconciliation, never from configure/apply code.
    /// Returns whether cancellation or an applied commit changed the scene.
    pub fn reconcile(&mut self, state: &State, output: &Output) -> bool {
        let Some(area) = state.space().output_geometry(output) else {
            let changed = !self.entries.is_empty();
            self.clear();
            return changed;
        };
        let signature = (
            output.clone(),
            area,
            output.current_scale().fractional_scale(),
            output.current_transform(),
        );
        let mut changed = false;
        if self.output.as_ref() != Some(&signature) || !state.resize_animations().enabled() {
            changed = !self.entries.is_empty();
            self.clear();
            self.output = Some(signature);
        }
        let now = Instant::now();
        self.entries.retain(|root, entry| {
            if !root.is_alive() {
                changed = true;
                return false;
            }
            if let Some(snapshot) = &entry.snapshot {
                snapshot.retire_inputs();
            }
            let Some(window) = state
                .visible_windows()
                .find(|w| w.toplevel().is_some_and(|top| top.wl_surface() == root))
            else {
                changed = true;
                return false;
            };
            let Some(target) = Geometry::of(state, window) else {
                changed = true;
                return false;
            };
            if let Some(timeline) = &entry.timeline {
                if entry.last_queued.progress >= 1.0 || target != timeline.target {
                    changed = true;
                    return false;
                }
            } else {
                let applied = with_states(root, |states| {
                    states
                        .data_map
                        .get::<XdgToplevelSurfaceData>()
                        .and_then(|role| role.lock().unwrap().current_serial)
                        .is_some_and(|current| current >= entry.serial)
                });
                // An applied role serial alone is insufficient while a layout
                // transaction still pins Space and allocation geometry.
                if applied && state.resize_displayed_frame(window).is_none() {
                    changed = true;
                    if target == entry.from {
                        return false;
                    }
                    entry.timeline = Some(Timeline::new(
                        entry.from,
                        target,
                        now,
                        state.resize_animations().duration(),
                    ));
                } else if now.saturating_duration_since(entry.captured) >= PENDING_LIFETIME {
                    changed = true;
                    return false;
                }
            }
            true
        });
        changed
    }
}
