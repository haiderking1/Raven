use super::State;

impl State {
    /// Coalesce visual changes until the backend runs after event dispatch.
    pub(crate) fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    pub(crate) fn take_redraw_request(&mut self) -> bool {
        std::mem::take(&mut self.redraw_requested)
    }
}
