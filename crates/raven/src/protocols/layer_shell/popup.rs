use crate::state::State;
use smithay::wayland::shell::{wlr_layer::LayerSurface, xdg::PopupSurface};

impl State {
    pub(super) fn position_layer_popup(&mut self, parent: &LayerSurface, popup: &PopupSurface) {
        if !self
            .layers
            .surfaces
            .iter()
            .any(|layer| layer.layer_surface() == parent)
        {
            popup.send_popup_done();
            return;
        }
        // Smithay sets the popup's parent before dispatching WlrLayerShellHandler::new_popup.
        // The XDG handler already tracked it while it was still parentless.
        let positioner = popup.with_pending_state(|state| state.positioner);
        self.position_popup(popup, positioner);
    }
}
