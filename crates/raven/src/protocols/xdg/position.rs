use crate::state::State;
use smithay::{
    desktop::{
        PopupKind, PopupManager, WindowSurfaceType, find_popup_root_surface,
        get_popup_toplevel_coords, layer_map_for_output,
    },
    wayland::{
        compositor::with_states,
        shell::xdg::{PopupSurface, PositionerState, XdgPopupSurfaceData},
    },
};

impl State {
    pub(crate) fn position_popup(&self, popup: &PopupSurface, positioner: PositionerState) {
        let kind = PopupKind::Xdg(popup.clone());
        let target = find_popup_root_surface(&kind).ok().and_then(|root| {
            let output = self.output.as_ref()?;
            let location = if let Some(window) = self
                .space()
                .elements()
                .find(|w| w.toplevel().is_some_and(|t| t.wl_surface() == &root))
            {
                self.space().element_location(window)?
            } else {
                let map = layer_map_for_output(output);
                let layer = map.layer_for_surface(&root, WindowSurfaceType::TOPLEVEL)?;
                let geometry = map.layer_geometry(layer)?;
                self.space().output_geometry(output)?.loc + geometry.loc - layer.bbox().loc
            };
            let mut target = self.space().output_geometry(output)?;
            // Positioners use the parent's window-geometry origin, not its buffer origin.
            target.loc -= location + get_popup_toplevel_coords(&kind);
            Some(target)
        });
        let geometry = target
            .map(|target| positioner.get_unconstrained_geometry(target))
            .unwrap_or_else(|| positioner.get_geometry());
        popup.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = geometry;
        });
    }

    /// Parent placement changes alter output constraints, not the client's anchor.
    /// Only committed reactive positioners permit unsolicited reconfiguration.
    pub(crate) fn refresh_reactive_popups(&self, index: usize) {
        if index != self.workspaces.active {
            return;
        }
        for window in self.visible_windows() {
            let Some(top) = window.toplevel() else {
                continue;
            };
            for (kind, _) in PopupManager::popups_for_surface(top.wl_surface()) {
                let PopupKind::Xdg(popup) = kind else {
                    continue;
                };
                let reactive = with_states(popup.wl_surface(), |states| {
                    states
                        .data_map
                        .get::<XdgPopupSurfaceData>()
                        .is_some_and(|data| data.lock().unwrap().current.positioner.reactive)
                });
                if !reactive || !popup.is_initial_configure_sent() {
                    continue;
                }
                let positioner = popup.with_pending_state(|state| state.positioner);
                if !positioner.reactive {
                    continue;
                }
                self.position_popup(&popup, positioner);
                if popup.send_pending_configure().is_err() {
                    popup.send_popup_done();
                }
            }
        }
    }
}
