use std::time::{Duration, Instant};

const PRESENTATION_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug)]
struct Pending<T> {
    ticket: T,
    submitted: Instant,
}

/// Two owned slots, not a growing queue. Only the KMS slot has a watchdog.
#[derive(Debug)]
pub(super) struct Flight<T> {
    pending: Option<Pending<T>>,
    queued: Option<T>,
}

impl<T> Flight<T> {
    pub fn new() -> Self {
        Self {
            pending: None,
            queued: None,
        }
    }

    pub fn pending(&self) -> Option<&T> {
        self.pending.as_ref().map(|pending| &pending.ticket)
    }

    pub fn queued(&self) -> Option<&T> {
        self.queued.as_ref()
    }

    pub fn submitted(&self) -> Option<Instant> {
        self.pending.as_ref().map(|pending| pending.submitted)
    }

    /// Returns whether the ticket occupies the KMS slot rather than waiting.
    pub fn push(&mut self, ticket: T, now: Instant) -> bool {
        assert!(self.queued.is_none(), "frame successor slot is full");
        if self.pending.is_none() {
            self.pending = Some(Pending {
                ticket,
                submitted: now,
            });
            true
        } else {
            self.queued = Some(ticket);
            false
        }
    }

    pub fn presented(&mut self, successor_submit_time: Instant) -> Option<T> {
        let completed = self.pending.take()?;
        self.pending = self.queued.take().map(|ticket| Pending {
            ticket,
            submitted: successor_submit_time,
        });
        Some(completed.ticket)
    }

    pub fn discard_pending(&mut self) -> Option<T> {
        // Valid only just after promotion, before another render can queue.
        assert!(
            self.queued.is_none(),
            "cannot discard with a waiting successor"
        );
        self.pending.take().map(|pending| pending.ticket)
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.submitted().map(|time| time + PRESENTATION_TIMEOUT)
    }

    pub fn reset(&mut self) {
        self.pending = None;
        self.queued = None;
    }
}
