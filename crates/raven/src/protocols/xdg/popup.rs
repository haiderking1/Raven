use crate::state::State;
use smithay::{
    desktop::{
        PopupKeyboardGrab, PopupKind, PopupPointerGrab, PopupUngrabStrategy,
        find_popup_root_surface,
    },
    input::pointer::Focus,
    reexports::wayland_server::protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    utils::Serial,
    wayland::shell::xdg::PopupSurface,
};

impl State {
    pub(crate) fn configure_popup(&mut self, surface: &WlSurface) {
        if let Some(PopupKind::Xdg(popup)) = self.popup_manager.find_popup(surface) {
            if !popup.is_initial_configure_sent() && popup.send_configure().is_err() {
                popup.send_popup_done();
            }
        }
    }

    pub(crate) fn grab_popup(&mut self, popup: PopupSurface, seat: WlSeat, serial: Serial) {
        if !self.seat.owns(&seat) {
            popup.send_popup_done();
            return;
        }
        let kind = PopupKind::Xdg(popup.clone());
        let Ok(root) = find_popup_root_surface(&kind) else {
            popup.send_popup_done();
            return;
        };
        let Some(keyboard) = self.seat.get_keyboard() else {
            popup.send_popup_done();
            return;
        };
        let Some(pointer) = self.seat.get_pointer() else {
            popup.send_popup_done();
            return;
        };
        // Never let an unfocused client take the seat by requesting a popup grab.
        let root_focused = keyboard.current_focus().is_some_and(|focus| {
            focus == root
                || self
                    .popup_manager
                    .find_popup(&focus)
                    .and_then(|p| find_popup_root_surface(&p).ok())
                    .as_ref()
                    == Some(&root)
        });
        if !root_focused || !self.popup_root_is_visible(&root) {
            popup.send_popup_done();
            return;
        }
        let Ok(mut grab) = self
            .popup_manager
            .grab_popup(root.clone(), kind, &self.seat, serial)
        else {
            return;
        };
        let previous = grab.previous_serial().unwrap_or(serial);
        if (keyboard.is_grabbed() && !keyboard.has_grab(serial) && !keyboard.has_grab(previous))
            || (pointer.is_grabbed() && !pointer.has_grab(serial) && !pointer.has_grab(previous))
        {
            grab.ungrab(PopupUngrabStrategy::All);
            return;
        }
        keyboard.set_focus(self, grab.current_grab(), serial);
        keyboard.set_grab(self, PopupKeyboardGrab::new(&grab), serial);
        pointer.set_grab(self, PopupPointerGrab::new(&grab), serial, Focus::Keep);
        self.popup_grab = Some((root, grab));
    }
}
