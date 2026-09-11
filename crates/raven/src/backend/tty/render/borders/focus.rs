use crate::state::State;
use smithay::{
    desktop::find_popup_root_surface, reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::get_parent,
};

/// Resolve once per scene, rather than scanning every window for each border.
pub(crate) fn active_root(state: &State) -> Option<WlSurface> {
    let mut root = state.seat.get_keyboard()?.current_focus()?;
    while let Some(parent) = get_parent(&root) {
        root = parent;
    }
    if let Some(popup) = state.popup_manager.find_popup(&root)
        && let Ok(parent) = find_popup_root_surface(&popup)
    {
        root = parent;
    }
    Some(root)
}
