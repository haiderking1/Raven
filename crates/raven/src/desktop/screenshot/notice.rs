use super::native;
use crate::state::State;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use std::time::Duration;
impl State {
    pub(crate) fn screenshot_notice(&mut self, message: &str) {
        eprintln!("raven: {message}");
        self.screenshot.notice = native::hud_pixels(message).map(|p| native::hud_buffer(&p));
        self.arm_screenshot_notice();
    }
    pub(super) fn arm_screenshot_notice(&mut self) {
        if let Some(handle) = &self.screenshot.handle {
            if let Some(token) = self.screenshot.notice_timer.take() {
                handle.remove(token);
            }
            self.screenshot.notice_timer = handle
                .insert_source(
                    Timer::from_duration(Duration::from_secs(5)),
                    |_, _, state| {
                        state.screenshot.notice = None;
                        state.screenshot.notice_timer = None;
                        state.request_redraw();
                        TimeoutAction::Drop
                    },
                )
                .ok();
        }
        self.request_redraw();
    }
}
