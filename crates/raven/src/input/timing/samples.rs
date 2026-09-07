use std::{fmt, time::Duration};

/// Fixed storage, even if the reporting timer is delayed indefinitely.
#[derive(Default)]
pub(super) struct Samples {
    count: u64,
    total_ns: u128,
    min: Duration,
    max: Duration,
}

impl Samples {
    pub fn add(&mut self, value: Duration) {
        if self.count == 0 || value < self.min {
            self.min = value;
        }
        self.max = self.max.max(value);
        self.count = self.count.saturating_add(1);
        self.total_ns = self.total_ns.saturating_add(value.as_nanos());
    }
}

impl fmt::Display for Samples {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.count == 0 {
            return f.write_str("0:n/a");
        }
        write!(
            f,
            "{}:{:.3}/{:.3}/{:.3}",
            self.count,
            self.min.as_secs_f64() * 1000.0,
            self.total_ns as f64 / self.count as f64 / 1_000_000.0,
            self.max.as_secs_f64() * 1000.0
        )
    }
}
