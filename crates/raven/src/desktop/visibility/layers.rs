use crate::state::State;
use smithay::{
    desktop::LayerSurface,
    utils::IsAlive,
    wayland::shell::wlr_layer::{KeyboardInteractivity, Layer},
};

impl State {
    pub(crate) fn layer_is_visible(&self, layer: &LayerSurface) -> bool {
        layer.alive()
            && self.layer_is_mapped(layer)
            && (self.fullscreen_window().is_none()
                || layer.layer() == Layer::Overlay
                || (layer.layer() == Layer::Top
                    && layer.cached_state().keyboard_interactivity
                        == KeyboardInteractivity::Exclusive))
    }
}
