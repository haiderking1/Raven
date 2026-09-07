use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cycle {
    Current,
    New,
}

#[derive(Debug, Default)]
pub(super) struct Callbacks {
    cycle: Option<Cycle>,
    deadline: Option<Instant>,
}

impl Callbacks {
    pub fn submitted(&mut self) {
        self.deadline = None;
        self.cycle = Some(Cycle::New);
    }

    pub fn empty(&mut self, now: Instant, next: Instant) {
        // Rendering and an expired timer can arrive in the same event batch.
        if self.expired(now) {
            self.cycle = Some(Cycle::New);
        }
        self.deadline = Some(next);
        self.current();
    }

    pub fn current(&mut self) {
        // A same-batch pageflip must never downgrade a submission's New cycle.
        self.cycle.get_or_insert(Cycle::Current);
    }

    fn expired(&self, now: Instant) -> bool {
        self.deadline.is_some_and(|deadline| now >= deadline)
    }

    pub fn due(&self, now: Instant) -> bool {
        self.cycle.is_some() || self.expired(now)
    }

    pub fn advance_cycle(&self, now: Instant) -> bool {
        self.cycle == Some(Cycle::New) || self.expired(now)
    }

    pub fn sent(&mut self, now: Instant) {
        self.cycle = None;
        if self.expired(now) {
            self.deadline = None;
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
}
