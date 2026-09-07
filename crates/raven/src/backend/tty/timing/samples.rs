use std::time::Duration;

/// Constant-space aggregation; recording a frame never allocates or sorts.
#[derive(Default)]
pub(super) struct Samples {
    pub count: u64,
    pub total: Duration,
    pub min: Duration,
    pub max: Duration,
}

impl Samples {
    pub fn add(&mut self, value: Duration) {
        self.min = if self.count == 0 {
            value
        } else {
            self.min.min(value)
        };
        self.max = self.max.max(value);
        self.total += value;
        self.count += 1;
    }

    pub fn mean_ms(&self) -> Option<f64> {
        (self.count != 0).then(|| self.total.as_secs_f64() * 1000.0 / self.count as f64)
    }
}
