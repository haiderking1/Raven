use super::super::{Placement, configure::bounded_size, hints::Hints};
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle, Size},
};

/// Restore size around the current tile center, never a saved screen position.
/// Hyprland makes an otherwise inconspicuous toggle 10 logical pixels larger.
pub(super) fn placement(
    window: &Window,
    frame: Rectangle<i32, Logical>,
    client: Rectangle<i32, Logical>,
    remembered: Option<Size<i32, Logical>>,
) -> Placement {
    let hints = Hints::committed(window);
    let desired = window.geometry().size;
    let fallback = if desired.w > 2 && desired.h > 2 {
        desired
    } else {
        (640, 400).into()
    };
    let mut size = remembered
        .filter(|size| size.w >= 5 && size.h >= 5)
        .unwrap_or_else(|| bounded_size(&hints, Some(fallback), None));
    if (i64::from(size.w) - i64::from(client.size.w)).abs() < 5
        && (i64::from(size.h) - i64::from(client.size.h)).abs() < 5
    {
        size.w = size.w.saturating_add(10);
        size.h = size.h.saturating_add(10);
    }
    let size = bounded_size(&hints, Some(size), None);
    let frame_size = size + (frame.size - client.size);
    let offset = Point::from((
        (f64::from(frame.size.w) - f64::from(frame_size.w)) / 2.0,
        (f64::from(frame.size.h) - f64::from(frame_size.h)) / 2.0,
    ));
    Placement {
        manual_size: true,
        position: Some((frame.loc.to_f64() + offset).to_i32_round()),
        natural: Some(size),
        hints: Some(hints),
        ..Placement::default()
    }
}
