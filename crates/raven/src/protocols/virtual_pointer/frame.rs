use smithay::{
    backend::input::{Axis, AxisSource},
    input::pointer::AxisFrame,
    reexports::wayland_server::{WEnum, protocol::wl_pointer},
};

#[derive(Default)]
pub(super) struct Pending {
    pub epoch: u64,
    pub source: Option<AxisSource>,
    pub frame: Option<AxisFrame>,
}
impl Pending {
    pub fn reset(&mut self, epoch: u64) {
        *self = Self {
            epoch,
            ..Default::default()
        };
    }
    pub fn axis(&mut self, time: u32, axis: Axis, value: f64, discrete: Option<i32>, stop: bool) {
        let mut frame = self.frame.take().unwrap_or_else(|| AxisFrame::new(time));
        frame.time = time;
        frame = frame.value(axis, value);
        if let Some(steps) = discrete {
            let old = frame.v120.unwrap_or_default();
            let old = match axis {
                Axis::Horizontal => old.0,
                Axis::Vertical => old.1,
            };
            frame = frame.v120(axis, old.saturating_add(steps.saturating_mul(120)));
        }
        if stop {
            frame = frame.stop(axis);
        }
        self.frame = Some(frame);
    }
    pub fn take(&mut self) -> Option<AxisFrame> {
        let source = self.source.take();
        self.frame.take().map(|mut frame| {
            frame.source = source;
            frame
        })
    }
}

pub(super) fn axis(value: WEnum<wl_pointer::Axis>) -> Option<Axis> {
    match value {
        WEnum::Value(wl_pointer::Axis::VerticalScroll) => Some(Axis::Vertical),
        WEnum::Value(wl_pointer::Axis::HorizontalScroll) => Some(Axis::Horizontal),
        _ => None,
    }
}
pub(super) fn source(value: WEnum<wl_pointer::AxisSource>) -> Option<AxisSource> {
    match value {
        WEnum::Value(wl_pointer::AxisSource::Wheel) => Some(AxisSource::Wheel),
        WEnum::Value(wl_pointer::AxisSource::Finger) => Some(AxisSource::Finger),
        WEnum::Value(wl_pointer::AxisSource::Continuous) => Some(AxisSource::Continuous),
        WEnum::Value(wl_pointer::AxisSource::WheelTilt) => Some(AxisSource::WheelTilt),
        _ => None,
    }
}
