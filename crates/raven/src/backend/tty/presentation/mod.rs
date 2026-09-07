mod clock;
mod sequence;

pub(super) use clock::completion;
use smithay::{
    backend::drm::DrmEventMetadata, desktop::utils::OutputPresentationFeedback,
    reexports::wayland_protocols::wp::presentation_time::server::wp_presentation_feedback::Kind,
    utils::Monotonic, wayland::presentation::Refresh,
};
use std::time::Duration;

#[derive(Default)]
pub(super) struct Presentation {
    sequence: sequence::Sequence,
}

impl Presentation {
    pub fn complete(
        &mut self,
        feedback: &mut OutputPresentationFeedback,
        metadata: Option<DrmEventMetadata>,
        completed: &clock::Completion,
        refresh_millihz: i32,
    ) {
        let mut flags = Kind::Vsync | Kind::HwCompletion;
        if completed.hardware_clock {
            flags |= Kind::HwClock;
        }
        let refresh = if refresh_millihz > 0 {
            Refresh::Fixed(Duration::from_nanos(
                1_000_000_000_000 / refresh_millihz as u64,
            ))
        } else {
            Refresh::Unknown
        };
        let sequence = self.sequence.observe(metadata.map(|meta| meta.sequence));
        feedback.presented::<_, Monotonic>(completed.time, refresh, sequence, flags);
    }
}
