use super::{Active, region::Area};
use crate::state::State;
use smithay::{
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Point},
    wayland::{
        compositor::get_parent,
        pointer_constraints::{PointerConstraint, with_pointer_constraint},
    },
};

pub(super) fn root(surface: &WlSurface) -> WlSurface {
    let mut root = surface.clone();
    while let Some(parent) = get_parent(&root) {
        root = parent;
    }
    root
}

impl State {
    /// No pointer-handle queries: this is also called from Smithay's locked motion hook.
    pub(crate) fn reconcile_pointer_capture(&mut self) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let Some((surface, _)) = self.input.capture.focus.clone() else {
            self.release_pointer_capture();
            return;
        };
        if !surface.is_alive() || self.input.capture.suspended {
            self.release_pointer_capture();
            return;
        }
        // The ordinary desktop path needs no additional scene scan when this
        // surface has never requested capture. Never nest surface-state locks.
        let Some(region) =
            with_pointer_constraint(&surface, &pointer, |c| c.map(|c| c.region().cloned()))
        else {
            self.release_pointer_capture();
            return;
        };
        let hit = self.surface_under(self.pointer_location);
        let eligible = (!self.input.capture.keyboard_seen
            || self
                .input
                .capture
                .keyboard_focus
                .as_ref()
                .is_some_and(|keyboard| root(keyboard) == root(&surface)))
            && hit.as_ref().is_some_and(|(hit, _)| hit == &surface);
        if !eligible {
            self.release_pointer_capture();
            return;
        }
        let (_, origin) = hit.unwrap();
        self.input.capture.focus = Some((surface.clone(), origin));
        if self
            .input
            .capture
            .active
            .as_ref()
            .is_some_and(|active| active.surface != surface)
        {
            self.release_pointer_capture();
        }
        let local = self.pointer_location - origin;
        let inside = Area::new(&surface, region).is_some_and(|area| area.contains(local));
        let mut ancestors = self
            .input
            .capture
            .active
            .as_mut()
            .map(|active| std::mem::take(&mut active.ancestors))
            .unwrap_or_default();
        ancestors.clear();
        ancestors.push(surface.clone());
        while let Some(parent) = get_parent(ancestors.last().unwrap()) {
            ancestors.push(parent);
        }
        let hint = self
            .input
            .capture
            .active
            .as_ref()
            .filter(|active| active.surface == surface)
            .and_then(|active| active.hint);
        let active = with_pointer_constraint(&surface, &pointer, |constraint| {
            let constraint = constraint?;
            if !inside {
                if constraint.is_active() {
                    constraint.deactivate();
                }
                return None;
            }
            if !constraint.is_active() {
                constraint.activate();
            }
            let locked = matches!(&*constraint, PointerConstraint::Locked(_));
            Some(Active {
                surface: surface.clone(),
                ancestors,
                origin,
                locked,
                hint,
            })
        });
        self.input.capture.active = active;
    }

    /// Forced release never restores client hints and never locks pointer internals.
    pub(crate) fn release_pointer_capture(&mut self) {
        let Some(active) = self.input.capture.active.take() else {
            return;
        };
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        // WlSurface data remains available during destruction hooks, even after the
        // resource stops being alive. The constraint resource may still need unlocked.
        with_pointer_constraint(&active.surface, &pointer, |constraint| {
            if let Some(constraint) = constraint.filter(|c| c.is_active()) {
                constraint.deactivate();
            }
        });
    }

    pub(crate) fn pointer_is_captured(&self) -> bool {
        self.input.capture.active.is_some()
    }

    pub(crate) fn pointer_is_locked(&self) -> bool {
        self.input
            .capture
            .active
            .as_ref()
            .is_some_and(|active| active.locked)
    }

    pub(crate) fn captured_pointer_location(
        &self,
        proposed: Point<f64, Logical>,
    ) -> Point<f64, Logical> {
        let Some(active) = &self.input.capture.active else {
            return proposed;
        };
        if active.locked {
            return self.pointer_location;
        }
        let Some(pointer) = self.seat.get_pointer() else {
            return self.pointer_location;
        };
        let region = with_pointer_constraint(&active.surface, &pointer, |c| {
            c.map(|c| c.region().cloned())
        });
        let confined = region
            .and_then(|region| Area::new(&active.surface, region))
            .map(|area| {
                active.origin
                    + area.confine(
                        self.pointer_location - active.origin,
                        proposed - active.origin,
                    )
            })
            .unwrap_or(self.pointer_location);
        // Clipping and stacking can further restrict the visible portion of this surface.
        if self
            .surface_under(confined)
            .as_ref()
            .is_some_and(|(surface, origin)| surface == &active.surface && origin == &active.origin)
        {
            confined
        } else {
            self.pointer_location
        }
    }

    pub(crate) fn pointer_capture_keyboard_focus(&mut self, focused: Option<&WlSurface>) {
        self.input.capture.keyboard_seen = true;
        self.input.capture.keyboard_focus = focused.cloned();
        self.reconcile_pointer_capture();
    }

    pub(crate) fn suspend_pointer_capture(&mut self) {
        self.input.capture.suspended = true;
        self.release_pointer_capture();
    }

    pub(crate) fn resume_pointer_capture(&mut self) {
        self.input.capture.suspended = false;
        // Re-hit-test on the next normal refresh or input event, not against stale VT state.
    }
}
