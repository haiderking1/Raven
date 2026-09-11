mod background;
mod coordination;
mod cycle;
mod surfaces;
mod visibility;
pub(crate) use cycle::FrameCallbacks;

use crate::state::State;
use smithay::desktop::utils::send_frames_surface_tree;
use std::time::Duration;

impl State {
    /// Begin a callback cycle after latching buffers or an estimated refresh.
    pub fn send_frames(&self, time: Duration) {
        self.frame_callbacks.advance();
        self.resend_frames(time);
    }

    /// Catch newly requested callbacks without waking a surface twice this cycle.
    pub(crate) fn resend_frames(&self, time: Duration) {
        let Some(output) = &self.output else {
            return;
        };
        let Some(geometry) = self.space().output_geometry(output) else {
            return;
        };
        self.send_layer_frames(time);
        for window in self.space().elements() {
            if self.window_is_visible(window)
                && self
                    .space()
                    .element_bbox(window)
                    .is_some_and(|bbox| bbox.overlaps(geometry))
            {
                window.send_frame(output, time, None, |_, states| {
                    self.frame_callbacks.output(states, output, time)
                });
            }
        }
        if let Some(icon) = &self.dnd_icon {
            send_frames_surface_tree(icon, output, time, None, |_, states| {
                self.frame_callbacks.output(states, output, time)
            });
        }
        if let smithay::input::pointer::CursorImageStatus::Surface(cursor) = &self.cursor_status {
            send_frames_surface_tree(cursor, output, time, None, |_, states| {
                self.frame_callbacks.output(states, output, time)
            });
        }
    }
}
