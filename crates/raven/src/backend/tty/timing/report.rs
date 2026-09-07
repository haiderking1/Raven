use super::{samples::Samples, window::Window};
use std::{fmt, time::Duration};

pub(in crate::backend::tty) struct Report {
    pub(super) span: Duration,
    pub(super) refresh_millihz: i32,
    pub(super) clock: &'static str,
    pub(super) window: Window,
}

impl fmt::Display for Samples {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mean_ms() {
            Some(mean) => write!(
                f,
                "{:.3}/{mean:.3}/{:.3}",
                self.min.as_secs_f64() * 1000.0,
                self.max.as_secs_f64() * 1000.0
            ),
            None => f.write_str("n/a"),
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = &self.window;
        write!(
            f,
            "raven: frame-timing span={:.3}s mode={:.3}Hz flips={} flip_hz={:.2} draws={} empty={} seq_steps=1:{},2:{},3+:{} clock={} draw_ms={} empty_ms={} flip_ms={} dispatch_ms={} timer_late_ms={} timer_early={} metadata_missing={} clock_discontinuities={}",
            self.span.as_secs_f64(),
            self.refresh_millihz as f64 / 1000.0,
            w.flips,
            w.flips as f64 / self.span.as_secs_f64(),
            w.draws.count,
            w.empty.count,
            w.steps[0],
            w.steps[1],
            w.steps[2],
            self.clock,
            w.draws,
            w.empty,
            w.flip_intervals,
            w.dispatch_delay,
            w.timer_lateness,
            w.timer_early,
            w.missing_metadata,
            w.clock_discontinuities,
        )?;
        write!(
            f,
            " seq_repeat={} seq_reset={} seq_last=",
            w.repeated_sequences, w.reset_sequences
        )?;
        match w.last_sequence {
            Some(sequence) => write!(f, "{sequence}"),
            None => f.write_str("n/a"),
        }
    }
}
