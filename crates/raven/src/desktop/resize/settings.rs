use super::Transactions;
use crate::state::State;

#[cfg(test)]
#[path = "settings/tests.rs"]
mod tests;

impl Transactions {
    fn configuration_ready(&self) -> bool {
        // `collecting` records whether the last outer batch collected windows.
        // It is not cleared by end_resize_batch; depth tracks active collection.
        self.depth == 0
            && self.held.is_empty()
            && self.live.is_empty()
            && self.batch.is_none()
            && !self.releasing
            && !self.suspended
    }
}

impl State {
    /// Do not supersede a held fullscreen/layout cohort or a live drag on reload.
    pub(crate) fn configuration_layout_ready(&self) -> bool {
        self.resize.configuration_ready() && !self.configuration_drag_active()
    }
}
