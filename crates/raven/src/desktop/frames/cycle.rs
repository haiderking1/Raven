use smithay::{
    output::{Output, WeakOutput},
    wayland::compositor::SurfaceData,
};
use std::{cell::Cell, sync::Mutex, time::Duration};

#[derive(Default)]
pub(crate) struct FrameCallbacks {
    pub(super) cycle: Cell<u64>,
    pub(super) occluded: Cell<bool>,
}

#[derive(Default)]
pub(super) struct SurfaceFrames(pub Mutex<SurfaceFrameState>);

#[derive(Default)]
pub(super) struct SurfaceFrameState {
    pub visibility: Option<(WeakOutput, bool)>,
    pub last_cycle: Option<u64>,
    pub last_time: Option<Duration>,
    pub coordination_pending: bool,
    pub coordination_until: Option<Duration>,
}

impl FrameCallbacks {
    pub fn advance(&self) {
        self.cycle.set(self.cycle.get().wrapping_add(1));
    }

    pub fn output(&self, states: &SurfaceData, output: &Output, time: Duration) -> Option<Output> {
        let frames = states.data_map.get_or_insert(SurfaceFrames::default);
        let mut frames = frames.0.lock().expect("surface frame state poisoned");
        if frames
            .visibility
            .as_ref()
            .is_some_and(|(owner, visible)| owner != output || !visible)
        {
            return None;
        }
        if frames.coordination_pending
            || frames.coordination_until.is_some_and(|until| time < until)
        {
            return None;
        }
        let cycle = self.cycle.get();
        if frames.last_cycle == Some(cycle) {
            return None;
        }
        frames.last_cycle = Some(cycle);
        frames.last_time = Some(time);
        Some(output.clone())
    }
}
