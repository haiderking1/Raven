use crate::state::State;
use smithay::{
    desktop::{PopupKeyboardGrab, PopupPointerGrab, Window, find_popup_root_surface},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::get_parent,
};

impl State {
    /// Evaluate the agreed visibility policy against prospective ownership. No
    /// callbacks run while this temporary value is installed.
    pub(super) fn fullscreen_hidden_roots(
        &mut self,
        index: usize,
        next: Option<&Window>,
    ) -> Vec<WlSurface> {
        if index != self.workspaces.active {
            return Vec::new();
        }
        let windows: Vec<_> = self
            .space()
            .elements()
            .filter(|w| self.window_is_visible(w))
            .cloned()
            .collect();
        let layers: Vec<_> = self
            .layers
            .surfaces
            .iter()
            .filter(|l| self.layer_is_visible(l))
            .cloned()
            .collect();
        let previous = std::mem::replace(
            &mut self.workspaces.entries[index].fullscreen.displayed,
            next.cloned(),
        );
        let mut hidden: Vec<_> = windows
            .iter()
            .filter(|w| !self.window_is_visible(w))
            .filter_map(|w| w.toplevel().map(|t| t.wl_surface().clone()))
            .collect();
        hidden.extend(
            layers
                .iter()
                .filter(|l| !self.layer_is_visible(l))
                .map(|l| l.wl_surface().clone()),
        );
        self.workspaces.entries[index].fullscreen.displayed = previous;
        hidden
    }

    pub(super) fn fullscreen_grab_blocks(&self, hidden: &[WlSurface]) -> bool {
        if hidden.is_empty() {
            return false;
        }
        // Popup cleanup unsets grabs by serial. A DnD grab can reuse the
        // initiating popup serial, so do not let cleanup release that grab.
        let dismissed_popup_serial = self
            .popup_grab
            .as_ref()
            .filter(|(root, _)| hidden.contains(root))
            .map(|(_, grab)| grab.serial());
        if let Some(pointer) = self.seat.get_pointer()
            && pointer.is_grabbed()
            && !pointer
                .with_grab(|_, grab| grab.is::<PopupPointerGrab<State>>())
                .unwrap_or(false)
        {
            if dismissed_popup_serial.is_some_and(|serial| pointer.has_grab(serial)) {
                return true;
            }
            let focus = pointer
                .grab_start_data()
                .and_then(|data| data.focus.map(|(surface, _)| surface));
            if focus
                .as_ref()
                .is_none_or(|surface| self.fullscreen_hides_surface(hidden, surface))
            {
                return true;
            }
        }
        if let Some(keyboard) = self.seat.get_keyboard()
            && keyboard.is_grabbed()
            && !keyboard
                .with_grab(|_, grab| grab.is::<PopupKeyboardGrab<State>>())
                .unwrap_or(false)
        {
            if dismissed_popup_serial.is_some_and(|serial| keyboard.has_grab(serial)) {
                return true;
            }
            let focus = keyboard.grab_start_data().and_then(|data| data.focus);
            if focus
                .as_ref()
                .is_none_or(|surface| self.fullscreen_hides_surface(hidden, surface))
            {
                return true;
            }
        }
        false
    }

    fn fullscreen_hides_surface(&self, hidden: &[WlSurface], surface: &WlSurface) -> bool {
        let mut root = surface.clone();
        while let Some(parent) = get_parent(&root) {
            root = parent;
        }
        if let Some(popup) = self.popup_manager.find_popup(&root)
            && let Ok(parent) = find_popup_root_surface(&popup)
        {
            root = parent;
        }
        hidden.contains(&root)
    }
}
