use super::geometry::Geometry;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Sample {
    pub geometry: Geometry,
    pub progress: f32,
}

pub(crate) struct Timeline {
    from: Geometry,
    pub target: Geometry,
    start: Instant,
    duration: Duration,
}

impl Timeline {
    pub fn new(from: Geometry, target: Geometry, start: Instant, duration: Duration) -> Self {
        Self {
            from,
            target,
            start,
            duration,
        }
    }

    pub fn sample(&self, now: Instant) -> Sample {
        let t = if self.duration.is_zero() {
            1.0
        } else {
            (now.saturating_duration_since(self.start).as_secs_f64() / self.duration.as_secs_f64())
                .clamp(0.0, 1.0)
        };
        // Bounded cubic ease-out: no overshoot, settling timer, or browser delay.
        let p = 1.0 - (1.0 - t).powi(3);
        Sample {
            geometry: self.from.interpolate(self.target, p),
            progress: p as f32,
        }
    }
}
