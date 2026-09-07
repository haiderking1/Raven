//! Fixed query lifecycle. Only an available result can return a slot to Free.

mod slot;
#[cfg(test)]
mod tests;

use super::driver::Queries;
use slot::{Interval, Slot, State};
use std::time::{Duration, Instant};

const CAPACITY: usize = 4;

pub(super) struct Pool {
    slots: [Slot; CAPACITY],
    bits: u32,
    active: Option<usize>,
    depth: u32,
    enabled: bool,
    lost: bool,
}

impl Pool {
    pub(super) fn new(gl: &impl Queries) -> Option<Self> {
        if gl.reset_detected() || gl.current() != 0 {
            return None;
        }
        let bits = gl.counter_bits();
        // The extension requires at least 30 bits for a nonzero counter.
        if !(30..=64).contains(&bits) {
            return None;
        }
        let mut ids = [0; CAPACITY];
        gl.generate(&mut ids);
        if gl.reset_detected() {
            return None;
        }
        if ids
            .iter()
            .enumerate()
            .any(|(i, id)| *id == 0 || ids[..i].contains(id))
        {
            gl.delete(&ids);
            return None;
        }
        // Establish our disjoint baseline before any measured interval.
        gl.disjoint();
        Some(Self {
            slots: ids.map(|id| Slot {
                id,
                state: State::Free,
            }),
            bits: bits as u32,
            active: None,
            depth: 0,
            enabled: true,
            lost: false,
        })
    }

    pub(super) fn enabled(&self) -> bool {
        self.enabled
    }

    pub(super) fn fault(&mut self) {
        self.enabled = false;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        for slot in &mut self.slots {
            slot.invalidate();
        }
    }

    /// A lost context owns retirement of its objects. Never use these names
    /// in a replacement context, even if EGL recycles its raw handle.
    fn lose(&mut self) {
        self.fault();
        self.lost = true;
        self.active = None;
        for slot in &mut self.slots {
            slot.id = 0;
            slot.state = State::Free;
        }
    }

    fn observe(&mut self, gl: &impl Queries) -> bool {
        if self.lost {
            return false;
        }
        if gl.reset_detected() {
            self.lose();
            return false;
        }
        if gl.disjoint() {
            self.invalidate();
            return false;
        }
        true
    }

    pub(super) fn begin(&mut self, gl: &impl Queries) {
        if self.depth != 0 {
            // Nested brackets cannot describe a single render job. Keep the
            // outer target owned until its matching end, but discard timing.
            self.depth = self
                .depth
                .checked_add(1)
                .expect("GPU timing bracket depth overflow");
            if let Some(index) = self.active {
                self.slots[index].invalidate();
            }
            return;
        }
        self.depth = 1;
        if !self.enabled || !self.observe(gl) || gl.current() != 0 {
            return;
        }
        let Some(index) = self
            .slots
            .iter()
            .position(|slot| matches!(slot.state, State::Free))
        else {
            // Backpressure drops measurement, never rendering or query results.
            return;
        };
        let interval = Interval {
            start: Instant::now(),
            valid: true,
        };
        gl.begin(self.slots[index].id);
        if gl.current() != self.slots[index].id {
            self.fault();
            return;
        }
        self.slots[index].state = State::Active(interval);
        self.active = Some(index);
    }

    pub(super) fn end(&mut self, gl: &impl Queries) {
        if self.depth == 0 {
            return;
        }
        self.depth -= 1;
        if self.depth != 0 {
            return;
        }
        self.observe(gl);
        if !self.lost {
            self.finish_active(gl);
        }
    }

    fn finish_active(&mut self, gl: &impl Queries) {
        let Some(index) = self.active.take() else {
            return;
        };
        let State::Active(mut interval) = self.slots[index].state else {
            unreachable!("active query must own an active slot");
        };
        if gl.current() == self.slots[index].id {
            gl.end();
            if gl.current() != 0 {
                // Preserve ownership for a later cleanup attempt.
                self.active = Some(index);
                self.fault();
                return;
            }
        } else {
            // Someone ended or replaced our query. Never end their query.
            interval.valid = false;
            self.fault();
        }
        self.slots[index].state = State::Pending(interval);
    }

    pub(super) fn sample(&mut self, gl: &impl Queries) -> Option<Duration> {
        if !self.enabled || !self.observe(gl) {
            return None;
        }
        let mut maximum = None;
        for slot in &mut self.slots {
            let State::Pending(interval) = slot.state else {
                continue;
            };
            if !gl.available(slot.id) {
                continue;
            }
            // Never issue RESULT before AVAILABLE. Invalid intervals still
            // occupy their names until completion, but need no result read.
            if interval.within_counter_range(self.bits) {
                let ns = gl.result(slot.id);
                let limit = (1u128 << self.bits) - 1;
                if (ns as u128) < limit && interval.within_counter_range(self.bits) {
                    maximum = Some(
                        maximum
                            .unwrap_or(Duration::ZERO)
                            .max(Duration::from_nanos(ns)),
                    );
                }
            }
            slot.state = State::Free;
        }
        // A disjoint/reset during polling invalidates even results just read.
        if !self.observe(gl) {
            return None;
        }
        maximum
    }

    pub(super) fn reset(&mut self, gl: &impl Queries) {
        self.observe(gl);
        self.invalidate();
        self.depth = 0;
        if !self.lost {
            self.finish_active(gl);
        }
        // Pending invalid queries are retired by availability polling, not
        // immediately reused. A lost/faulted collector stays disabled.
    }

    pub(super) fn destroy(&mut self, gl: &impl Queries) {
        self.reset(gl);
        if !self.lost {
            // GL permits deletion of pending queries. This does not retrieve
            // results or reuse their storage; the driver retires that storage.
            gl.delete(&self.slots.map(|slot| slot.id));
        }
        self.lose();
        self.depth = 0;
    }
}
