use crate::state::State;
use smithay::{
    desktop::{PopupManager, PopupUngrabStrategy},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::SERIAL_COUNTER,
};

impl State {
    pub(crate) fn dismiss_window_popups(&mut self, root: &WlSurface) {
        if self
            .popup_grab
            .as_ref()
            .is_some_and(|(parent, _)| parent == root)
        {
            self.release_popup_grab();
        }
        let popups: Vec<_> = PopupManager::popups_for_surface(root)
            .map(|(p, _)| p)
            .collect();
        for popup in popups {
            let _ = PopupManager::dismiss_popup(root, &popup);
        }
    }

    pub(crate) fn refresh_popup_grab(&mut self) {
        if self
            .popup_grab
            .as_ref()
            .is_some_and(|(_, grab)| grab.has_ended())
        {
            self.release_popup_grab();
        }
    }

    fn release_popup_grab(&mut self) {
        let Some((_, mut grab)) = self.popup_grab.take() else {
            return;
        };
        grab.ungrab(PopupUngrabStrategy::All);
        let serial = grab.serial();
        if let Some(keyboard) = self.seat.get_keyboard() {
            if keyboard.has_grab(serial) {
                keyboard.unset_grab(self);
            }
        }
        if let Some(pointer) = self.seat.get_pointer() {
            if pointer.has_grab(serial) {
                let time = self.start_time.elapsed().as_millis() as u32;
                pointer.unset_grab(self, SERIAL_COUNTER.next_serial(), time);
            }
        }
    }
}
