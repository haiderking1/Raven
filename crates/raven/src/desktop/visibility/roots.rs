use crate::state::State;
use smithay::{
    desktop::{WindowSurfaceType, layer_map_for_output},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

impl State {
    pub(crate) fn popup_root_is_visible(&self, root: &WlSurface) -> bool {
        if let Some(window) = self.space().elements().find(|window| {
            window
                .toplevel()
                .is_some_and(|toplevel| toplevel.wl_surface() == root)
        }) {
            return self.window_is_visible(window);
        }
        let Some(output) = self.output.as_ref() else {
            return false;
        };
        layer_map_for_output(output)
            .layer_for_surface(root, WindowSurfaceType::TOPLEVEL)
            .is_some_and(|layer| self.layer_is_visible(layer))
    }
}
