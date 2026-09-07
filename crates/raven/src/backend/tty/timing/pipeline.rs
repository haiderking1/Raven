use super::Timing;
use std::time::Duration;

impl Timing {
    pub fn queue_placement(&mut self, queued: bool, deferred: bool) {
        if queued {
            if deferred {
                self.window.software_queued += 1;
            } else {
                self.kms_submitted();
            }
        }
    }

    pub fn kms_submitted(&mut self) {
        self.window.kms_submitted += 1;
    }

    pub fn gpu_sample(&mut self, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.window.gpu_elapsed.add(duration);
        }
    }
}
