use super::QueuedFeedback;

/// Kept outside Smithay as well as inside its queue: a failed successor submit
/// must not destroy the completion feedback for the frame that just flipped.
pub(in crate::backend::tty) struct Frame {
    pub feedback: QueuedFeedback,
    pub primary_scanout: bool,
    pub hardware_cursor: bool,
}

impl Frame {
    pub fn uses_planes(&self) -> bool {
        self.primary_scanout || self.hardware_cursor
    }
}
