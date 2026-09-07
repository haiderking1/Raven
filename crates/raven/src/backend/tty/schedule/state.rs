use super::clock::FrameClock;
use std::time::{Duration, Instant};

const PRESENTATION_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Callbacks {
    CurrentCycle,
    NewCycle,
}

/// One KMS frame in flight, with client production overlapping presentation.
#[derive(Debug)]
pub struct Schedule {
    clock: FrameClock,
    active: bool,
    redraw: bool,
    pending: Option<Instant>,
    callback_deadline: Option<Instant>,
    callbacks: Option<Callbacks>,
}

impl Schedule {
    pub fn new(refresh_millihz: i32, now: Instant) -> Self {
        Self {
            clock: FrameClock::new(refresh_millihz, now),
            active: true,
            redraw: true,
            pending: None,
            callback_deadline: None,
            callbacks: None,
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn request_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn render_due(&self) -> bool {
        self.active && self.pending.is_none() && self.redraw
    }

    pub fn rendered(&mut self, queued: bool, now: Instant) {
        if !self.render_due() {
            return;
        }
        // Preserve an elapsed no-damage cycle even if a commit and its timer
        // became ready in the same event batch.
        if self
            .callback_deadline
            .is_some_and(|deadline| now >= deadline)
        {
            self.callbacks = Some(Callbacks::NewCycle);
        }
        self.redraw = false;
        if queued {
            self.pending = Some(now);
            self.callback_deadline = None;
            // Buffers are now latched. Clients may produce the next buffers,
            // but render_due remains false until this frame completes.
            self.callbacks = Some(Callbacks::NewCycle);
        } else {
            self.callback_deadline = Some(self.clock.next(now));
            self.callbacks.get_or_insert(Callbacks::CurrentCycle);
        }
    }

    /// The caller maps the DRM timestamp into Instant's domain. Unknown kernel
    /// timestamps use dispatch time without claiming hardware-clock accuracy.
    pub fn presented(&mut self, time: Instant) -> bool {
        if !self.active || self.pending.take().is_none() {
            return false;
        }
        self.clock.presented(time);
        self.callbacks = Some(Callbacks::CurrentCycle);
        true
    }

    pub fn callbacks_due(&self, now: Instant) -> bool {
        self.active
            && (self.callbacks.is_some()
                || self
                    .callback_deadline
                    .is_some_and(|deadline| now >= deadline))
    }

    pub fn callbacks_advance_cycle(&self, now: Instant) -> bool {
        self.callbacks == Some(Callbacks::NewCycle)
            || self
                .callback_deadline
                .is_some_and(|deadline| now >= deadline)
    }

    pub fn callbacks_sent(&mut self, now: Instant) {
        self.callbacks = None;
        if self
            .callback_deadline
            .is_some_and(|deadline| now >= deadline)
        {
            self.callback_deadline = None;
        }
    }

    /// Idle has no timer. No-damage callbacks get one predicted refresh wake.
    pub fn deadline(&self) -> Option<Instant> {
        if !self.active {
            return None;
        }
        self.pending
            .map(|queued| queued + PRESENTATION_TIMEOUT)
            .into_iter()
            .chain(self.callback_deadline)
            .min()
    }

    pub fn stalled(&self, now: Instant) -> bool {
        self.active
            && self
                .pending
                .is_some_and(|queued| now >= queued + PRESENTATION_TIMEOUT)
    }

    pub fn pause(&mut self) {
        self.active = false;
        self.pending = None;
        self.callback_deadline = None;
        self.callbacks = None;
        self.redraw = false;
    }

    pub fn resume(&mut self, now: Instant) {
        self.clock.reset(now);
        self.active = true;
        self.pending = None;
        self.callback_deadline = None;
        self.callbacks = None;
        self.redraw = true;
    }
}
