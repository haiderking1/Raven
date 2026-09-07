use super::{InputTiming, samples::Samples};
use smithay::backend::{
    input::{Event, InputEvent},
    libinput::LibinputInputBackend,
};
use std::time::Duration;

pub(super) const KINDS: [&str; 10] = [
    "keyboard",
    "motion",
    "absolute",
    "button",
    "axis",
    "touch_down",
    "touch_motion",
    "touch_up",
    "touch_cancel",
    "touch_frame",
];

#[derive(Default)]
pub(super) struct Dispatch {
    pub observed: u64,
    pub missing: u64,
    pub future: u64,
    pub clock_unavailable: u64,
    pub age: Samples,
}

/// Smithay's libinput Event::time is u64 microseconds in CLOCK_MONOTONIC.
/// Never use time_msec, which truncates and wraps, or State::start_time.
fn timestamp(event: &InputEvent<LibinputInputBackend>) -> Option<(usize, u64)> {
    Some(match event {
        InputEvent::Keyboard { event } => (0, event.time()),
        InputEvent::PointerMotion { event } => (1, event.time()),
        InputEvent::PointerMotionAbsolute { event } => (2, event.time()),
        InputEvent::PointerButton { event } => (3, event.time()),
        InputEvent::PointerAxis { event } => (4, event.time()),
        InputEvent::TouchDown { event } => (5, event.time()),
        InputEvent::TouchMotion { event } => (6, event.time()),
        InputEvent::TouchUp { event } => (7, event.time()),
        InputEvent::TouchCancel { event } => (8, event.time()),
        InputEvent::TouchFrame { event } => (9, event.time()),
        _ => return None,
    })
}

impl InputTiming {
    /// Observe source arrival before routing. Touch observations do not imply
    /// touch protocol delivery, which Raven's current routing does not implement.
    pub fn observe_event(&mut self, event: &InputEvent<LibinputInputBackend>) {
        let Some((kind, micros)) = timestamp(event) else {
            return;
        };
        let now = self.monotonic();
        let dispatch = &mut self.window.dispatch[kind];
        dispatch.observed = dispatch.observed.saturating_add(1);
        let source = if micros == 0 {
            dispatch.missing = dispatch.missing.saturating_add(1);
            None
        } else if let Some(now) = now {
            let time = Duration::from_micros(micros);
            if let Some(age) = now.checked_sub(time) {
                dispatch.age.add(age);
                Some(time)
            } else {
                dispatch.future = dispatch.future.saturating_add(1);
                None
            }
        } else {
            dispatch.clock_unavailable = dispatch.clock_unavailable.saturating_add(1);
            None
        };
        self.unflushed.add(source, now);
        self.unqueued.add(source, now);
    }
}
