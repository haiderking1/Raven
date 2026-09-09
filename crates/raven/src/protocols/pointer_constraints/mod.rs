mod surface;
use crate::state::State;
use smithay::{
    delegate_pointer_constraints,
    input::pointer::PointerHandle,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::pointer_constraints::{PointerConstraint, PointerConstraintsHandler},
};

impl PointerConstraintsHandler for State {
    fn new_constraint(&mut self, surface: &WlSurface, _pointer: &PointerHandle<Self>) {
        surface::install(surface);
        self.reconcile_pointer_capture();
    }

    fn cursor_position_hint(
        &mut self,
        surface: &WlSurface,
        _pointer: &PointerHandle<Self>,
        location: Point<f64, Logical>,
    ) {
        self.pointer_capture_hint(surface, location);
    }

    fn constraint_committed(&mut self, _surface: &WlSurface, _pointer: &PointerHandle<Self>) {
        self.reconcile_pointer_capture();
    }

    fn constraint_destroyed(
        &mut self,
        surface: &WlSurface,
        _pointer: &PointerHandle<Self>,
        constraint: PointerConstraint,
    ) {
        if constraint.is_active() {
            self.pointer_capture_destroyed(surface);
        }
    }
}

delegate_pointer_constraints!(State);
