use crate::state::State;
use smithay::{
    desktop::{LayerSurface, WindowSurfaceType, layer_map_for_output},
    utils::SERIAL_COUNTER,
    wayland::shell::wlr_layer::{KeyboardInteractivity, Layer},
};

impl State {
    pub(crate) fn exclusive_keyboard_layer(&self) -> Option<LayerSurface> {
        let map = layer_map_for_output(self.output.as_ref()?);
        [Layer::Overlay, Layer::Top].into_iter().find_map(|level| {
            map.layers_on(level)
                .rev()
                .find(|layer| {
                    layer.cached_state().keyboard_interactivity == KeyboardInteractivity::Exclusive
                })
                .cloned()
        })
    }

    pub(crate) fn focus_layer(&mut self, layer: &LayerSurface) {
        if !layer.can_receive_keyboard_focus() {
            return;
        }
        if self
            .seat
            .get_keyboard()
            .and_then(|keyboard| keyboard.current_focus())
            .as_ref()
            == Some(layer.wl_surface())
        {
            return;
        }
        for window in self.space().elements() {
            window.set_activated(false);
            if let Some(toplevel) = window.toplevel() {
                toplevel.send_pending_configure();
            }
        }
        if let Some(keyboard) = self.seat.get_keyboard() {
            let focus = layer.wl_surface().clone();
            if keyboard.current_focus().as_ref() != Some(&focus) {
                keyboard.set_focus(self, Some(focus), SERIAL_COUNTER.next_serial());
            }
        }
    }

    /// Exclusive top/overlay layers outrank windows; OnDemand layers need a click.
    /// Return whether a mapped layer currently owns the keyboard.
    pub(crate) fn refresh_layer_focus(&mut self) -> bool {
        if let Some(layer) = self.exclusive_keyboard_layer() {
            if let Some((root, _)) = &self.popup_grab
                && root != layer.wl_surface()
            {
                let root = root.clone();
                self.dismiss_window_popups(&root);
            }
            let grabbed = self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
                || self.seat.get_pointer().is_some_and(|p| p.is_grabbed());
            if !grabbed {
                self.focus_layer(&layer);
                return true;
            }
        }
        let Some(output) = &self.output else {
            return false;
        };
        let Some(focus) = self.seat.get_keyboard().and_then(|k| k.current_focus()) else {
            return false;
        };
        layer_map_for_output(output)
            .layer_for_surface(&focus, WindowSurfaceType::ALL)
            .is_some_and(|layer| layer.can_receive_keyboard_focus())
    }
}
