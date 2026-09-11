use crate::state::State;
use smithay::{
    desktop::{Window, WindowSurfaceType},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

pub(crate) struct WindowHit<'a> {
    pub(crate) window: &'a Window,
    pub(crate) surface: WlSurface,
    pub(crate) origin: Point<f64, Logical>,
}

enum WindowTarget<'a> {
    Surface(WindowHit<'a>),
    Border(&'a Window),
}

impl State {
    #[cfg(test)]
    pub(crate) fn window_under(&self, point: Point<f64, Logical>) -> Option<WindowHit<'_>> {
        match self.window_target_under(point)? {
            WindowTarget::Surface(hit) => Some(hit),
            WindowTarget::Border(_) => None,
        }
    }

    pub(crate) fn window_focus_under(&self, point: Point<f64, Logical>) -> Option<&Window> {
        match self.window_target_under(point)? {
            WindowTarget::Surface(hit) => Some(hit.window),
            WindowTarget::Border(window) => Some(window),
        }
    }

    fn window_target_under(&self, point: Point<f64, Logical>) -> Option<WindowTarget<'_>> {
        let output = self
            .output
            .as_ref()
            .and_then(|output| self.space().output_geometry(output));
        if output.is_some_and(|area| !area.to_f64().contains(point)) {
            return None;
        }
        for window in self.visible_windows().rev() {
            let Some(origin) = self.window_surface_origin(window) else {
                continue;
            };
            let local = point - origin.to_f64();
            // Popups escape the allocated tile, but never the output.
            let popup = window.surface_under(
                local,
                WindowSurfaceType::POPUP | WindowSurfaceType::SUBSURFACE,
            );
            let hit = popup.or_else(|| {
                // Without an output there is no allocation to clip against.
                // Keep logical surface-tree hit testing available during setup
                // and output teardown; actual surface input regions still apply.
                self.window_layout_geometry(window)
                    .map_or(output.is_none(), |area| area.to_f64().contains(point))
                    .then(|| {
                        window.surface_under(
                            local,
                            WindowSurfaceType::TOPLEVEL | WindowSurfaceType::SUBSURFACE,
                        )
                    })
                    .flatten()
            });
            if let Some((surface, offset)) = hit {
                return Some(WindowTarget::Surface(WindowHit {
                    window,
                    surface,
                    origin: (origin + offset).to_f64(),
                }));
            }
            if self
                .window_frame_geometry(window)
                .is_some_and(|frame| frame.to_f64().contains(point))
                && self
                    .window_client_geometry(window)
                    .is_some_and(|client| !client.to_f64().contains(point))
            {
                // A compositor border blocks lower windows/layers but has no
                // wl_surface. Focus the owner without inventing content input.
                return Some(WindowTarget::Border(window));
            }
        }
        None
    }

    pub fn surface_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        if let Some(hit) = self.layer_under(point, &[Layer::Overlay, Layer::Top]) {
            return Some((hit.surface, hit.origin));
        }
        match self.window_target_under(point) {
            Some(WindowTarget::Surface(hit)) => return Some((hit.surface, hit.origin)),
            Some(WindowTarget::Border(_)) => return None,
            None => {}
        }
        self.layer_under(point, &[Layer::Bottom, Layer::Background])
            .map(|hit| (hit.surface, hit.origin))
    }
}
