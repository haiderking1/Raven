use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Physical, Rectangle},
};

/// Content excludes both compositor borders and client pixels outside the
/// committed window geometry. Buffer dimensions alone do not describe a window.
pub(super) fn bounds(
    state: &State,
    window: &Window,
    area: Rectangle<i32, Logical>,
    scale: f64,
) -> Option<Rectangle<i32, Physical>> {
    Some(super::paint::physical(
        state.window_render_geometry(window)?,
        area,
        scale,
    ))
}

pub(super) fn interpolate(
    from: Rectangle<i32, Physical>,
    to: Rectangle<i32, Physical>,
    progress: f64,
) -> Rectangle<i32, Physical> {
    let mix =
        |a: i32, b: i32| (f64::from(a) + (f64::from(b) - f64::from(a)) * progress).round() as i32;
    Rectangle::from_extremities(
        (mix(from.loc.x, to.loc.x), mix(from.loc.y, to.loc.y)),
        (
            mix(from.loc.x + from.size.w, to.loc.x + to.size.w),
            mix(from.loc.y + from.size.h, to.loc.y + to.size.h),
        ),
    )
}
