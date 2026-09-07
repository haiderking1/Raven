mod commit;
mod popup;

use crate::state::State;
use smithay::{
    delegate_layer_shell,
    desktop::LayerSurface as DesktopLayerSurface,
    output::Output,
    reexports::{
        wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1,
        wayland_server::{
            Resource,
            protocol::{wl_output::WlOutput, wl_surface::WlSurface},
        },
    },
    wayland::shell::{
        wlr_layer::{
            Layer, LayerSurface, LayerSurfaceConfigure, WlrLayerShellHandler, WlrLayerShellState,
        },
        xdg::PopupSurface,
    },
};

impl WlrLayerShellHandler for State {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        requested_output: Option<WlOutput>,
        layer: Layer,
        namespace: String,
    ) {
        let output = match requested_output {
            Some(resource) => Output::from_resource(&resource),
            None => self.output.clone(),
        };
        let Some(output) = output.filter(|output| {
            self.output.as_ref() == Some(output) && output.current_mode().is_some()
        }) else {
            surface.send_close();
            return;
        };
        if commit::already_buffered(surface.wl_surface()) {
            surface.shell_surface().post_error(
                zwlr_layer_surface_v1::Error::InvalidSurfaceState,
                "a new layer surface must not already have a buffer",
            );
            return;
        }
        commit::install(surface.wl_surface());
        self.layers
            .register(DesktopLayerSurface::new(surface, namespace), output, layer);
    }

    fn layer_destroyed(&mut self, surface: LayerSurface) {
        self.remove_layer(surface.wl_surface());
    }

    fn ack_configure(&mut self, surface: WlSurface, configure: LayerSurfaceConfigure) {
        if !self.layers.acknowledge(&surface, configure.serial) {
            if let Some(layer) = self
                .layers
                .surfaces
                .iter()
                .find(|layer| layer.wl_surface() == &surface)
            {
                layer.layer_surface().shell_surface().post_error(
                    zwlr_layer_surface_v1::Error::InvalidSurfaceState,
                    "configure serial belongs to an earlier layer mapping cycle",
                );
            }
        }
    }

    fn new_popup(&mut self, parent: LayerSurface, popup: PopupSurface) {
        self.position_layer_popup(&parent, &popup);
    }
}

delegate_layer_shell!(State);
