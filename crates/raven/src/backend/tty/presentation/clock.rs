use smithay::{
    backend::drm::{DrmEventMetadata, DrmEventTime},
    utils::{Clock, Monotonic},
};
use std::time::{Duration, Instant};

pub(in crate::backend::tty) struct Completion {
    pub time: Duration,
    pub instant: Instant,
    pub hardware_clock: bool,
}

pub(in crate::backend::tty) fn completion(metadata: Option<DrmEventMetadata>) -> Completion {
    let now: Duration = Clock::<Monotonic>::new().now().into();
    let instant = Instant::now();
    let kernel = metadata.and_then(|meta| match meta.time {
        DrmEventTime::Monotonic(time) if !time.is_zero() => Some(time),
        _ => None,
    });
    let time = kernel.unwrap_or(now);
    // Instant and CLOCK_MONOTONIC have unrelated Rust epochs. Map by a delta,
    // never reinterpret a kernel duration as elapsed compositor uptime.
    let mapped = if time >= now {
        instant.checked_add(time - now)
    } else {
        instant.checked_sub(now - time)
    };
    Completion {
        time,
        instant: mapped.unwrap_or(instant),
        hardware_clock: kernel.is_some(),
    }
}
