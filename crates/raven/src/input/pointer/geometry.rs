use smithay::utils::{Logical, Point, Rectangle};

/// The final pixel is a valid location, but the exclusive output edge is not.
pub(super) fn clamp(
    location: Point<f64, Logical>,
    bounds: Rectangle<i32, Logical>,
) -> Point<f64, Logical> {
    let min = bounds.loc.to_f64();
    let max: Point<f64, Logical> = (
        min.x + f64::from(bounds.size.w.saturating_sub(1).max(0)),
        min.y + f64::from(bounds.size.h.saturating_sub(1).max(0)),
    )
        .into();
    (
        location.x.clamp(min.x, max.x),
        location.y.clamp(min.y, max.y),
    )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_preserves_last_pixel_and_respects_output_origin() {
        let bounds = Rectangle::new((40, -20).into(), (1920, 1080).into());
        assert_eq!(
            clamp((1960.0, 1060.0).into(), bounds),
            (1959.0, 1059.0).into()
        );
        assert_eq!(clamp((-1.0, -30.0).into(), bounds), (40.0, -20.0).into());
        let edge = (1959.0, 1059.0).into();
        assert_eq!(clamp(edge, bounds), edge);
        let interior = (100.25, 200.75).into();
        assert_eq!(clamp(interior, bounds), interior);
    }
}
