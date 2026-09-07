use super::samples::Samples;

/// Counters for one reporting interval, separate from cross-report flip history.
#[derive(Default)]
pub(super) struct Window {
    pub draws: Samples,
    pub empty: Samples,
    pub flip_intervals: Samples,
    pub dispatch_delay: Samples,
    pub timer_lateness: Samples,
    pub flips: u64,
    pub timer_early: u64,
    pub steps: [u64; 3],
    pub missing_metadata: u64,
    pub clock_discontinuities: u64,
    pub repeated_sequences: u64,
    pub reset_sequences: u64,
    pub last_sequence: Option<u32>,
}
