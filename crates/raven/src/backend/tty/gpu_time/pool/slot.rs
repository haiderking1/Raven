use std::time::Instant;

#[derive(Clone, Copy)]
pub(super) enum State {
    Free,
    Active(Interval),
    Pending(Interval),
}

#[derive(Clone, Copy)]
pub(super) struct Interval {
    pub(super) start: Instant,
    pub(super) valid: bool,
}

impl Interval {
    pub(super) fn within_counter_range(self, bits: u32) -> bool {
        // CPU age is only an overflow guard, never a replacement GPU sample.
        // It includes queueing before begin and time until result collection.
        self.valid && self.start.elapsed().as_nanos() < (1u128 << bits) - 1
    }
}

#[derive(Clone, Copy)]
pub(super) struct Slot {
    pub(super) id: u32,
    pub(super) state: State,
}

impl Slot {
    pub(super) fn invalidate(&mut self) {
        match &mut self.state {
            State::Active(interval) | State::Pending(interval) => interval.valid = false,
            State::Free => {}
        }
    }
}
