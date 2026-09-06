use crate::state::State;
use smithay::desktop::utils::send_frames_surface_tree;
use std::time::Duration;

impl State {
    /// Call after submitting a frame, using elapsed time since start_time.
    pub fn send_frames(&self, time: Duration) {
        let Some(output) = &self.output else {
            return;
        };
        let Some(geometry) = self.space().output_geometry(output) else {
            return;
        };
        for window in self.space().elements() {
            if self
                .space()
                .element_bbox(window)
                .is_some_and(|bbox| bbox.overlaps(geometry))
            {
                window.send_frame(output, time, None, |_, _| Some(output.clone()));
            }
        }
        if let Some(icon) = &self.dnd_icon {
            send_frames_surface_tree(icon, output, time, None, |_, _| Some(output.clone()));
        }
        if let smithay::input::pointer::CursorImageStatus::Surface(cursor) = &self.cursor_status {
            send_frames_surface_tree(cursor, output, time, None, |_, _| Some(output.clone()));
        }
    }
}
