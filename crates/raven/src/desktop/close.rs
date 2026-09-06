use crate::state::State;

impl State {
    pub(crate) fn close_focused_window(&self) {
        let Some(focus) = self
            .seat
            .get_keyboard()
            .and_then(|keyboard| keyboard.current_focus())
        else {
            return;
        };
        let window = self.space.elements().find(|window| {
            // A popup grab can put keyboard focus on a popup or its subsurface.
            // Close the owning toplevel, not whichever tile the pointer is over.
            let mut owns_focus = false;
            window.with_surfaces(|surface, _| owns_focus |= surface == &focus);
            owns_focus
        });
        if let Some(toplevel) = window.and_then(|window| window.toplevel()) {
            // Let the client handle unsaved work and destroy its own surface.
            // Normal unmap/destroy handling will retile and restore focus.
            toplevel.send_close();
        }
    }
}
