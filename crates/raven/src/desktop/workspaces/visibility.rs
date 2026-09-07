use crate::state::State;
use smithay::{
    desktop::find_popup_root_surface, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

impl State {
    /// Unknown ownership remains conservatively repaintable. Layer/cursor trees
    /// are output-wide; only a known inactive window owner can suppress a redraw.
    pub(crate) fn surface_on_hidden_workspace(&self, root: &WlSurface) -> bool {
        let popup_root = self
            .popup_manager
            .find_popup(root)
            .and_then(|popup| find_popup_root_surface(&popup).ok());
        let root = popup_root.as_ref().unwrap_or(root);
        self.windows
            .iter()
            .find(|window| {
                window
                    .toplevel()
                    .is_some_and(|top| top.wl_surface() == root)
            })
            .and_then(|window| self.workspaces.index_of(window))
            .is_some_and(|owner| owner != self.workspaces.active)
    }
}
