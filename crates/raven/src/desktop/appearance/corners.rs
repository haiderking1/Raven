use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle},
};
/// Visual reference radius; not an asserted platform API constant.
pub(crate) const RADIUS: f32 = 16.0;
pub(crate) fn contains(
    rect: Rectangle<i32, Logical>,
    radius: f64,
    point: Point<f64, Logical>,
) -> bool {
    if !rect.to_f64().contains(point) {
        return false;
    }
    let r = radius
        .min(f64::from(rect.size.w.min(rect.size.h)) / 2.0)
        .max(0.0);
    let center: Point<f64, Logical> = (
        f64::from(rect.loc.x) + f64::from(rect.size.w) / 2.0,
        f64::from(rect.loc.y) + f64::from(rect.size.h) / 2.0,
    )
        .into();
    let x = ((point.x - center.x).abs() - f64::from(rect.size.w) / 2.0 + r).max(0.0);
    let y = ((point.y - center.y).abs() - f64::from(rect.size.h) / 2.0 + r).max(0.0);
    x * x + y * y <= r * r
}
impl State {
    pub(crate) fn window_corner_contains(
        &self,
        window: &Window,
        rect: Rectangle<i32, Logical>,
        point: Point<f64, Logical>,
        client: bool,
    ) -> bool {
        if self.applied_fullscreen_allocation(window).is_some() {
            return rect.to_f64().contains(point);
        }
        let inset = if client {
            self.window_frame_geometry(window)
                .map_or(0, |frame| (rect.loc.x - frame.loc.x).max(0))
        } else {
            0
        };
        contains(rect, (f64::from(RADIUS) - f64::from(inset)).max(0.0), point)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corners_reject_cutouts_but_keep_edges_centers_and_small_windows() {
        let rect = Rectangle::new((100, 200).into(), (80, 60).into());
        assert!(!contains(rect, 16.0, (101.0, 201.0).into()));
        assert!(contains(rect, 16.0, (116.0, 200.0).into()));
        assert!(contains(rect, 16.0, (140.0, 230.0).into()));
        assert!(!contains(rect, 16.0, (180.0, 230.0).into()));
        assert!(contains(rect, 0.0, (100.0, 200.0).into()));
        let small = Rectangle::from_size((8, 8).into());
        assert!(contains(small, 16.0, (4.0, 4.0).into()));
        assert!(!contains(small, 16.0, (0.0, 0.0).into()));
    }
}
