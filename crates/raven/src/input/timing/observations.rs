use super::samples::Samples;
use std::time::Duration;

#[derive(Clone, Copy)]
struct Range {
    oldest: Duration,
    latest: Duration,
}

fn include(range: &mut Option<Range>, time: Option<Duration>) {
    let Some(time) = time else { return };
    match range {
        Some(range) => {
            range.oldest = range.oldest.min(time);
            range.latest = range.latest.max(time);
        }
        None => {
            *range = Some(Range {
                oldest: time,
                latest: time,
            })
        }
    }
}

/// Observations between boundaries, with no claim about client buffer contents.
#[derive(Clone, Copy, Default)]
pub(super) struct Observations {
    pub count: u64,
    pub valid_sources: u64,
    source: Option<Range>,
    dispatch: Option<Range>,
}

impl Observations {
    pub fn add(&mut self, source: Option<Duration>, dispatch: Option<Duration>) {
        self.count = self.count.saturating_add(1);
        self.valid_sources = self
            .valid_sources
            .saturating_add(u64::from(source.is_some()));
        include(&mut self.source, source);
        include(&mut self.dispatch, dispatch);
    }
}

#[derive(Default)]
pub(super) struct Ages {
    pub batches: u64,
    pub observations: u64,
    pub valid_sources: u64,
    pub source_oldest: Samples,
    pub source_latest: Samples,
    pub dispatch_oldest: Samples,
    pub dispatch_latest: Samples,
    pub negative: u64,
}

impl Ages {
    pub fn record(&mut self, observations: Observations, boundary: Option<Duration>) {
        if observations.count == 0 {
            return;
        }
        self.batches = self.batches.saturating_add(1);
        self.observations = self.observations.saturating_add(observations.count);
        self.valid_sources = self
            .valid_sources
            .saturating_add(observations.valid_sources);
        let Some(boundary) = boundary else { return };
        for (range, oldest, latest) in [
            (
                observations.source,
                &mut self.source_oldest,
                &mut self.source_latest,
            ),
            (
                observations.dispatch,
                &mut self.dispatch_oldest,
                &mut self.dispatch_latest,
            ),
        ] {
            if let Some(range) = range {
                for (time, samples) in [(range.oldest, oldest), (range.latest, latest)] {
                    if let Some(age) = boundary.checked_sub(time) {
                        samples.add(age);
                    } else {
                        self.negative = self.negative.saturating_add(1);
                    }
                }
            }
        }
    }
}
