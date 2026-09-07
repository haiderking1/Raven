use smithay::desktop::utils::OutputPresentationFeedback;
use std::{cell::Cell, rc::Rc};

/// Queue first, then attach the accepted render's feedback on the same event
/// thread. A rejected plane attempt must not consume requests or ZeroCopy flags
/// that belong to its successful composition replacement.
#[derive(Clone, Default)]
pub(in crate::backend::tty) struct QueuedFeedback(Rc<Cell<Option<OutputPresentationFeedback>>>);

impl QueuedFeedback {
    pub fn set(&self, feedback: OutputPresentationFeedback) {
        self.0.set(Some(feedback));
    }
    pub fn take(&self) -> Option<OutputPresentationFeedback> {
        self.0.take()
    }
}
