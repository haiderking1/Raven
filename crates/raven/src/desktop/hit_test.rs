use crate::state::State;
use smithay::{
    desktop::WindowSurfaceType,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

impl State {
    pub fn surface_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        if let Some(hit) = self.layer_under(point, &[Layer::Overlay, Layer::Top]) {
            return Some((hit.surface, hit.origin));
        }
        if let Some((window, origin)) = self.space().element_under(point)
            && let Some((surface, offset)) =
                window.surface_under(point - origin.to_f64(), WindowSurfaceType::ALL)
        {
            return Some((surface, (origin + offset).to_f64()));
        }
        self.layer_under(point, &[Layer::Bottom, Layer::Background])
            .map(|hit| (hit.surface, hit.origin))
    }
}
