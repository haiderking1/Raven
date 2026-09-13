use crate::state::State;
use smithay::{
    desktop::{WindowSurfaceType, layer_map_for_output},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::Serial,
};

impl State {
    /// Non-keyboard-interactive panels can open grabbed menus from a click.
    /// Match the most recent delivered click, even after button release. Hover
    /// alone is not authorization, and an accepted grab consumes the click.
    pub(super) fn layer_popup_has_click(&self, root: &WlSurface, serial: Serial) -> bool {
        if self
            .exclusive_keyboard_layer()
            .is_some_and(|layer| layer.wl_surface() != root)
        {
            return false;
        }
        let Some(output) = self.output.as_ref() else {
            return false;
        };
        let Some(focus) = self.popup_click_focus(serial) else {
            return false;
        };
        let map = layer_map_for_output(output);
        let Some(layer) = map.layer_for_surface(root, WindowSurfaceType::TOPLEVEL) else {
            return false;
        };
        self.layer_is_visible(layer)
            && map
                .layer_for_surface(focus, WindowSurfaceType::ALL)
                .is_some_and(|clicked| clicked.wl_surface() == root)
    }
}
