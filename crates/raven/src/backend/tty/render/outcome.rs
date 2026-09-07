/// Accepted queue outcomes, not claims of completed display presentation.
#[derive(Default)]
pub(in crate::backend::tty) struct RenderOutcome {
    pub queued: bool,
    pub primary_scanout: bool,
    pub hardware_cursor: bool,
    pub recovered: bool,
}
