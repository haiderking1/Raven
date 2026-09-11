use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

impl State {
    /// Clip rendering to committed XDG geometry as well as the allocation.
    /// Firefox can attach an oversized buffer before acknowledging a fullscreen
    /// change. Those pixels must not leak into the scene or resize snapshots.
    pub(crate) fn window_render_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        let origin = self.window_surface_origin(window)?;
        let geometry = window.geometry();
        Rectangle::new(origin + geometry.loc, geometry.size)
            .intersection(self.window_client_geometry(window)?)
    }
}
