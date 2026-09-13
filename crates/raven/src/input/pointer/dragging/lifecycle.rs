use crate::state::State;
use smithay::utils::{Clock, Monotonic, SERIAL_COUNTER, Serial};

impl State {
    pub(crate) fn cancel_window_drag(&mut self) {
        let Some(drag) = self.input.drag.take() else {
            return;
        };
        drag.finish_resize(self);
        if let Some(pointer) = self.seat.get_pointer() {
            if pointer.has_grab(drag.serial) {
                pointer.unset_grab(
                    self,
                    SERIAL_COUNTER.next_serial(),
                    Clock::<Monotonic>::new().now().as_millis(),
                );
                pointer.frame(self);
            }
        }
        self.refresh_tiling_pointer();
        self.request_redraw();
    }

    pub(crate) fn reconcile_window_drag(&mut self) {
        if let Some(window) = self.input.resize_cleanup.take() {
            super::resizing::set_resizing(self, &window, false);
        }
        if self.input.drag.as_ref().is_some_and(|drag| {
            !drag.valid(self)
                || self.exclusive_keyboard_layer().is_some()
                || self
                    .seat
                    .get_keyboard()
                    .is_some_and(|keyboard| keyboard.is_grabbed())
        }) {
            self.cancel_window_drag();
        }
    }

    pub(super) fn finish_window_drag(&mut self, serial: Serial, time: u32) {
        let Some(drag) = self.input.drag.take() else {
            return;
        };
        let target = drag.target(self, self.pointer_location);
        drag.finish_resize(self);
        if let Some(pointer) = self.seat.get_pointer() {
            if pointer.has_grab(drag.serial) {
                pointer.unset_grab(self, serial, time);
                pointer.frame(self);
            }
        }
        if let Some(target) = target {
            self.swap_dragged_tiles(&drag.window, &target);
        }
        self.refresh_reactive_popups(self.workspaces.active);
        self.refresh_tiling_pointer();
        self.request_redraw();
    }
}
