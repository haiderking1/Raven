use crate::state::State;
use smithay::{desktop::Window, utils::SERIAL_COUNTER};

impl State {
    pub(crate) fn activate_window(&mut self, window: Option<Window>) {
        self.workspaces.entries[self.workspaces.active].focused = window.clone();
        let exclusive = self.exclusive_keyboard_layer();
        if let Some(window) = &window {
            self.space_mut().raise_element(window, false);
        }
        for candidate in self.space().elements() {
            candidate.set_activated(exclusive.is_none() && window.as_ref() == Some(candidate));
            if let Some(toplevel) = candidate.toplevel() {
                toplevel.send_pending_configure();
            }
        }
        if exclusive.is_some() {
            self.refresh_layer_focus();
            return;
        }
        let focus = window.and_then(|w| w.toplevel().map(|t| t.wl_surface().clone()));
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.set_focus(self, focus, SERIAL_COUNTER.next_serial());
        }
    }

    pub(crate) fn restore_focus(&mut self) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        if self.refresh_layer_focus() || keyboard.is_grabbed() {
            return;
        }
        if self.focused_window().is_none() {
            let workspace = &self.workspaces.entries[self.workspaces.active];
            let next = workspace
                .focused
                .as_ref()
                .filter(|window| workspace.space.element_location(window).is_some())
                .cloned()
                .or_else(|| workspace.space.elements().next_back().cloned());
            if next.is_some() || keyboard.current_focus().is_some() {
                self.activate_window(next);
            }
        }
    }
}
