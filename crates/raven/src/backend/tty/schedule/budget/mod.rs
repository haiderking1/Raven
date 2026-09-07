use std::time::Duration;

const INITIAL_COMPONENT: Duration = Duration::from_millis(1);
const SCHEDULING_MARGIN: Duration = Duration::from_micros(500);

/// Independent fast-attack, slow-release estimates of CPU and GPU elapsed work.
#[derive(Debug)]
pub(super) struct Budget {
    period: Duration,
    cpu: Duration,
    gpu: Duration,
    overlap: bool,
}

impl Budget {
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            cpu: INITIAL_COMPONENT.min(period),
            gpu: INITIAL_COMPONENT.min(period),
            overlap: false,
        }
    }

    pub fn observe_render(&mut self, cpu: Duration) {
        Self::observe(&mut self.cpu, cpu.min(self.period));
        self.pressure();
    }

    pub fn observe_gpu(&mut self, gpu: Duration) {
        Self::observe(&mut self.gpu, gpu.min(self.period));
        self.pressure();
    }

    fn observe(estimate: &mut Duration, sample: Duration) {
        if sample >= *estimate {
            *estimate = sample;
        } else {
            *estimate -= (*estimate - sample) / 16;
        }
    }

    fn pressure(&mut self) {
        let lead = self.lead();
        // Hysteresis: avoid switching queue policy on every noisy observation.
        if lead >= self.period * 3 / 4 {
            self.overlap = true;
        } else if lead <= self.period / 2 {
            self.overlap = false;
        }
    }

    pub fn needs_overlap(&self) -> bool {
        self.overlap
    }

    pub fn lead(&self) -> Duration {
        self.cpu
            .saturating_add(self.gpu)
            .saturating_add(SCHEDULING_MARGIN)
            .min(self.period)
    }
}
