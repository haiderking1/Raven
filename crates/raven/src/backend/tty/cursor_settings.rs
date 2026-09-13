use super::{PreparedCursor, TtyBackend};
impl TtyBackend {
    pub(crate) fn set_cursor_settings(&mut self, prepared: PreparedCursor) {
        self.scene.cursor = super::render::cursor::Cursors::prepared(prepared);
        self.schedule.request_redraw();
    }
}
