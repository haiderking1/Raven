//! Clock-free state transitions for one KMS frame and one software successor.
use super::observations::Observations;
use std::time::Duration;

#[cfg(test)]
mod tests;

pub(super) struct Snapshot {
    pub observations: Observations,
    /// Successful queue acceptance, including acceptance into software storage.
    pub queued: Option<Duration>,
    /// Clock read after actual KMS submission, never software acceptance.
    pub submitted: Option<Duration>,
}

pub(super) enum Pending {
    Snapshot(Snapshot),
    /// Attribution was lost. Keep a marker until the pipeline drains or resets.
    Ambiguous,
}

#[derive(Default)]
pub(super) struct Queue {
    pending: Option<Pending>,
    successor: Option<Snapshot>,
}

pub(super) struct Acceptance {
    pub invariant_mismatch: bool,
    pub discarded: u64,
}

pub(super) struct Completion {
    pub presented: Option<Pending>,
    /// Original acceptance clock and actual submission clock of the promotion.
    pub promotion: Option<(Option<Duration>, Option<Duration>)>,
    pub successor_discarded: bool,
    pub invariant_mismatch: bool,
}

impl Queue {
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn has_successor(&self) -> bool {
        self.successor.is_some()
    }

    pub fn is_ambiguous(&self) -> bool {
        matches!(self.pending, Some(Pending::Ambiguous))
    }

    pub fn accept(
        &mut self,
        observations: Observations,
        now: Option<Duration>,
        deferred: bool,
    ) -> Acceptance {
        let valid = if deferred {
            matches!(self.pending, Some(Pending::Snapshot(_))) && self.successor.is_none()
        } else {
            self.pending.is_none() && self.successor.is_none()
        };
        if !valid {
            // Do not overwrite or guess which of the accepted frames will flip.
            let discarded = 1
                + u64::from(matches!(self.pending, Some(Pending::Snapshot(_))))
                + u64::from(self.successor.is_some());
            self.pending = Some(Pending::Ambiguous);
            self.successor = None;
            return Acceptance {
                invariant_mismatch: true,
                discarded,
            };
        }
        let snapshot = Snapshot {
            observations,
            queued: now,
            submitted: if deferred { None } else { now },
        };
        if deferred {
            self.successor = Some(snapshot);
        } else {
            self.pending = Some(Pending::Snapshot(snapshot));
        }
        Acceptance {
            invariant_mismatch: false,
            discarded: 0,
        }
    }

    pub fn complete(&mut self, now: Option<Duration>, successor_submitted: bool) -> Completion {
        // Metadata belongs to this frame even when submitting its successor fails.
        let presented = self.pending.take();
        let successor = self.successor.take();
        let mut completion = Completion {
            presented,
            promotion: None,
            successor_discarded: false,
            invariant_mismatch: false,
        };
        match (successor, successor_submitted) {
            (Some(mut snapshot), true) => {
                snapshot.submitted = now;
                completion.promotion = Some((snapshot.queued, snapshot.submitted));
                self.pending = Some(Pending::Snapshot(snapshot));
            }
            (Some(_), false) => completion.successor_discarded = true,
            (None, true) => {
                completion.invariant_mismatch = true;
                self.pending = Some(Pending::Ambiguous);
            }
            (None, false) => {}
        }
        completion
    }
}
