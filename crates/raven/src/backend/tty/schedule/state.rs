use super::{Policy, budget::Budget, callbacks::Callbacks, clock::FrameClock, flight::Flight};
use std::time::{Duration, Instant};

/// Owns one KMS-pending ticket and, under Deadline, at most one successor.
#[derive(Debug)]
pub struct Schedule<T> {
    clock: FrameClock,
    budget: Budget,
    policy: Policy,
    flight: Flight<T>,
    callbacks: Callbacks,
    active: bool,
    redraw: bool,
}

impl<T> Schedule<T> {
    /// Compatibility constructor. Production callers should select a policy.
    #[cfg(test)]
    pub fn new(refresh_millihz: i32, now: Instant) -> Self {
        Self::with_policy(refresh_millihz, now, Policy::Immediate)
    }

    pub fn with_policy(refresh_millihz: i32, now: Instant, policy: Policy) -> Self {
        let clock = FrameClock::new(refresh_millihz, now);
        Self {
            budget: Budget::new(clock.interval()),
            clock,
            policy,
            flight: Flight::new(),
            callbacks: Callbacks::default(),
            active: true,
            redraw: true,
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn request_redraw(&mut self) {
        self.redraw = true;
    }

    fn successor_deadline(&self) -> Option<Instant> {
        let overlap = match self.policy {
            Policy::Immediate => false,
            Policy::Deadline => true,
            Policy::Adaptive => self.budget.needs_overlap(),
        };
        if !overlap || !self.redraw || self.queued().is_some() {
            return None;
        }
        self.flight.submitted().map(|submitted| {
            // Fix the target to this submission, not the moving dispatch time.
            // If the prediction is missed, the window remains open.
            self.clock.next(submitted) - self.budget.lead()
        })
    }

    pub fn render_due(&self, now: Instant) -> bool {
        self.active
            && self.redraw
            && (self.pending().is_none()
                || self
                    .successor_deadline()
                    .is_some_and(|deadline| now >= deadline))
    }

    /// Record a completed render. None means no damage, not a pending frame.
    /// The caller must have checked render_due before starting the render.
    /// now is the submission time for an initial ticket, or render completion
    /// time for a deferred successor. Report samples after recording the frame.
    pub fn rendered(&mut self, ticket: Option<T>, now: Instant) {
        if !self.render_due(now) {
            return;
        }
        self.redraw = false;
        match ticket {
            Some(ticket) => {
                if self.flight.push(ticket, now) {
                    self.callbacks.submitted();
                }
            }
            None => self.callbacks.empty(now, self.clock.next(now)),
        }
    }

    /// Complete the KMS slot and promote the successor. The caller maps kernel
    /// timestamps to Instant, using dispatch time when hardware time is unknown.
    /// Submit the promoted pending ticket immediately, without another render.
    /// Its watchdog starts at successor_submit_time, never at kernel_time or
    /// the earlier render time. Unexpected pageflips do not change the clock.
    pub fn presented(&mut self, kernel_time: Instant, successor_submit_time: Instant) -> Option<T> {
        if !self.active {
            return None;
        }
        let completed = self.flight.presented(successor_submit_time)?;
        self.clock.presented(kernel_time);
        if self.pending().is_some() {
            // Preparation did not wake this producer; successful KMS handoff does.
            self.callbacks.submitted();
        } else {
            self.callbacks.current();
        }
        Some(completed)
    }

    pub fn pending(&self) -> Option<&T> {
        self.flight.pending()
    }

    pub fn queued(&self) -> Option<&T> {
        self.flight.queued()
    }

    /// Only for immediate failure to submit the successor just promoted by
    /// presented. Never use for a watchdog timeout or an accepted KMS commit.
    /// Return ownership to the caller and request a fresh render for retry.
    pub fn discard_pending(&mut self) -> Option<T> {
        let ticket = self.flight.discard_pending();
        if ticket.is_some() {
            self.request_redraw();
        }
        ticket
    }

    /// Render wall time, including required driver and fallback sync waits.
    pub fn observe_render(&mut self, cpu: Duration) {
        self.budget.observe_render(cpu);
    }

    /// GPU timer interval, possibly including dependency or command-stream stalls.
    /// Do not pass CPU submission-to-fence latency or block awaiting a sample.
    pub fn observe_gpu(&mut self, gpu: Duration) {
        self.budget.observe_gpu(gpu);
    }

    pub fn callbacks_due(&self, now: Instant) -> bool {
        self.active && self.callbacks.due(now)
    }

    pub fn callbacks_advance_cycle(&self, now: Instant) -> bool {
        self.callbacks.advance_cycle(now)
    }

    pub fn callbacks_sent(&mut self, now: Instant) {
        self.callbacks.sent(now);
    }

    /// Only outstanding work has a timer. Immediate rendering needs no timer.
    pub fn deadline(&self) -> Option<Instant> {
        if !self.active {
            return None;
        }
        self.flight
            .deadline()
            .into_iter()
            .chain(self.callbacks.deadline())
            .chain(self.successor_deadline())
            .min()
    }

    pub fn stalled(&self, now: Instant) -> bool {
        self.active
            && self
                .flight
                .deadline()
                .is_some_and(|deadline| now >= deadline)
    }

    pub fn pause(&mut self) {
        self.active = false;
        self.flight.reset();
        self.callbacks = Callbacks::default();
        self.redraw = false;
    }

    pub fn resume(&mut self, now: Instant) {
        self.clock.reset(now);
        self.budget = Budget::new(self.clock.interval());
        self.flight.reset();
        self.callbacks = Callbacks::default();
        self.active = true;
        self.redraw = true;
    }
}
