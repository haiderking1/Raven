/// Accepted queue outcomes, not claims of completed display presentation.
#[derive(Default)]
pub(in crate::backend::tty) struct RenderOutcome {
    pub queued: bool,
    pub feedback: Option<super::super::presentation::QueuedFeedback>,
    pub primary_scanout: bool,
    pub hardware_cursor: bool,
    pub recovered: bool,
}
