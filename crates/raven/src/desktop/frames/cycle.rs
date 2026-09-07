use smithay::{output::Output, wayland::compositor::SurfaceData};
use std::{cell::Cell, sync::Mutex};

#[derive(Default)]
pub(crate) struct FrameCallbacks {
    cycle: Cell<u64>,
}

#[derive(Default)]
struct LastCallback(Mutex<Option<u64>>);

impl FrameCallbacks {
    pub fn advance(&self) {
        self.cycle.set(self.cycle.get().wrapping_add(1));
    }

    pub fn output(&self, states: &SurfaceData, output: &Output) -> Option<Output> {
        let last = states.data_map.get_or_insert(LastCallback::default);
        let mut last = last.0.lock().expect("frame callback cycle poisoned");
        let cycle = self.cycle.get();
        if *last == Some(cycle) {
            return None;
        }
        *last = Some(cycle);
        Some(output.clone())
    }
}
