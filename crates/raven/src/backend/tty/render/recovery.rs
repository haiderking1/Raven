use smithay::backend::drm::{DrmError, compositor::FrameError};
use std::{error::Error, fmt};

/// queue_frame only submits when Smithay has no pending frame. A successful
/// queue (including one parked behind a pending flip) must never be retried.
pub(super) fn plane_rejection<A, B, F>(error: &FrameError<A, B, F>) -> bool
where
    A: Error + Send + Sync + 'static,
    B: Error + Send + Sync + 'static,
    F: Error + Send + Sync + 'static,
{
    match error {
        FrameError::DrmError(DrmError::Access(access)) => matches!(
            access.source.raw_os_error(),
            Some(libc::EINVAL | libc::ERANGE | libc::ENOSPC)
        ),
        FrameError::DrmError(
            DrmError::UnsupportedPlaneConfiguration(_) | DrmError::TestFailed(_),
        ) => true,
        // In particular: EBUSY/EAGAIN, device loss, inactivity, and allocation
        // or synchronization failures are not evidence of a rejected plane.
        _ => false,
    }
}

#[derive(Debug)]
struct RecoveryFailure {
    rejection: String,
    fallback: Box<dyn Error>,
}

impl fmt::Display for RecoveryFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "plane submission rejected: {}; composition recovery failed: {}",
            self.rejection, self.fallback
        )
    }
}

impl Error for RecoveryFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.fallback.as_ref())
    }
}

pub(super) fn failure(
    original: &Option<String>,
    error: impl Into<Box<dyn Error>>,
) -> Box<dyn Error> {
    let fallback = error.into();
    match original {
        Some(rejection) => Box::new(RecoveryFailure {
            rejection: rejection.clone(),
            fallback,
        }),
        None => fallback,
    }
}
