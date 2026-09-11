use smithay::utils::{Physical, Rectangle, Scale};

/// Map both edges with the same origin and scale. Rounding an offset and a
/// width independently can leave adjoining edges a pixel apart.
pub(super) fn project(
    surface: Rectangle<i32, Physical>,
    window: Rectangle<i32, Physical>,
    target: Rectangle<i32, Physical>,
) -> Rectangle<i32, Physical> {
    let ratio = Scale {
        x: f64::from(target.size.w) / f64::from(window.size.w),
        y: f64::from(target.size.h) / f64::from(window.size.h),
    };
    let start = (surface.loc - window.loc)
        .to_f64()
        .upscale(ratio)
        .to_i32_round()
        + target.loc;
    let end = (surface.loc + surface.size - window.loc)
        .to_f64()
        .upscale(ratio)
        .to_i32_round()
        + target.loc;
    Rectangle::from_extremities(start, end)
}
