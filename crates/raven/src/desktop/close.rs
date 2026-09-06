use crate::state::State;

impl State {
    pub(crate) fn close_focused_window(&self) {
        if let Some(window) = self.focused_window()
            && let Some(toplevel) = window.toplevel()
        {
            // Let the client handle unsaved work and destroy its own surface.
            // Normal unmap/destroy handling will retile and restore focus.
            toplevel.send_close();
        }
    }
}
