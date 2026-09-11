use super::super::TtyBackend;

impl TtyBackend {
    pub(in crate::backend::tty) fn pause_frames(&mut self) {
        // Drop animation textures and copy fences before pausing/dropping EGL.
        self.cancel_resize_animations();
        // Smithay retains its queue across pause. Explicitly empty both shared
        // carriers now, before dropping our scheduler tickets.
        for frame in self
            .schedule
            .pending()
            .into_iter()
            .chain(self.schedule.queued())
        {
            drop(frame.feedback.take());
        }
        self.schedule.pause();
        self.deferred_recovery = None;
    }
}
