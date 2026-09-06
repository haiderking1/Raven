use smithay::{
    backend::{
        input::{Axis, AxisSource, PointerAxisEvent},
        libinput::LibinputInputBackend,
    },
    input::pointer::AxisFrame,
};

pub(super) fn frame(event: &impl PointerAxisEvent<LibinputInputBackend>) -> AxisFrame {
    let source = event.source();
    let mut frame = AxisFrame::new(event.time_msec()).source(source);
    for axis in [Axis::Horizontal, Axis::Vertical] {
        frame = frame.relative_direction(axis, event.relative_direction(axis));
        frame = add_axis(
            frame,
            axis,
            source,
            event.amount(axis),
            event.amount_v120(axis),
        );
    }
    frame
}

fn add_axis(
    mut frame: AxisFrame,
    axis: Axis,
    source: AxisSource,
    amount: Option<f64>,
    v120: Option<f64>,
) -> AxisFrame {
    // 120 units represent one wheel detent, not 120 discrete steps. Smithay
    // emits value120 or accumulates axis_discrete for older pointer versions.
    if let Some(v120) = v120 {
        frame = frame.v120(axis, v120.round() as i32);
    }
    if let Some(value) = amount.or_else(|| v120.map(|value| value * 15.0 / 120.0)) {
        if value != 0.0 {
            frame = frame.value(axis, value);
        } else if source == AxisSource::Finger && amount == Some(0.0) {
            // An absent axis is not a stopped axis.
            frame = frame.stop(axis);
        }
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_keeps_high_resolution_units_and_only_stops_present_finger_axes() {
        let wheel = add_axis(
            AxisFrame::new(1),
            Axis::Vertical,
            AxisSource::Wheel,
            None,
            Some(30.0),
        );
        assert_eq!(wheel.axis, (0.0, 3.75));
        assert_eq!(wheel.v120, Some((0, 30)));
        assert_eq!(wheel.stop, (false, false));

        let finger = add_axis(
            AxisFrame::new(2),
            Axis::Horizontal,
            AxisSource::Finger,
            None,
            None,
        );
        let finger = add_axis(finger, Axis::Vertical, AxisSource::Finger, Some(0.0), None);
        assert_eq!(finger.stop, (false, true));
        assert_eq!(finger.v120, None);
    }
}
