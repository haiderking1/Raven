use crate::state::State;
use smithay::desktop::layer_map_for_output;
use std::time::Duration;

impl State {
    pub(crate) fn send_layer_frames(&self, time: Duration) {
        let Some(output) = &self.output else {
            return;
        };
        for layer in layer_map_for_output(output).layers() {
            layer.send_frame(output, time, None, |_, states| {
                self.frame_callbacks.output(states, output, time)
            });
        }
    }
}
