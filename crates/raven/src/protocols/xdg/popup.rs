use crate::state::State;
use smithay::{
    desktop::{
        PopupKeyboardGrab, PopupKind, PopupPointerGrab, PopupUngrabStrategy,
        find_popup_root_surface, get_popup_toplevel_coords,
    },
    input::pointer::Focus,
    reexports::wayland_server::protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    utils::Serial,
    wayland::shell::xdg::{PopupSurface, PositionerState},
};

impl State {
    pub(crate) fn configure_popup(&mut self, surface: &WlSurface) {
        if let Some(PopupKind::Xdg(popup)) = self.popup_manager.find_popup(surface) {
            if !popup.is_initial_configure_sent() && popup.send_configure().is_err() {
                popup.send_popup_done();
            }
        }
    }

    pub(crate) fn position_popup(&self, popup: &PopupSurface, positioner: PositionerState) {
        let kind = PopupKind::Xdg(popup.clone());
        let target = find_popup_root_surface(&kind).ok().and_then(|root| {
            let window = self
                .windows
                .iter()
                .find(|w| w.toplevel().is_some_and(|t| t.wl_surface() == &root))?;
            let location = self.space().element_location(window)?;
            let output = self.output.as_ref()?;
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
        if !root_focused {
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
