use super::Appearance;
use smithay::utils::{Logical, Point, Rectangle};

/// Reduce opposing gaps proportionally, keeping at least one client pixel.
fn fit_pair(first: i32, second: i32, extent: i32) -> (i32, i32) {
    let available = i64::from(extent.saturating_sub(1).max(0));
    let sum = i64::from(first) + i64::from(second);
    if sum <= available {
        return (first, second);
    }
    if sum == 0 {
        return (0, 0);
    }
    let first = i64::from(first) * available / sum;
    (first as i32, (available - first) as i32)
}

impl Appearance {
    pub(crate) fn inset_workarea(&self, area: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
        let (left, right) = fit_pair(self.outer.left, self.outer.right, area.size.w);
        let (top, bottom) = fit_pair(self.outer.top, self.outer.bottom, area.size.h);
        Rectangle::new(
            (
                area.loc.x.saturating_add(left),
                area.loc.y.saturating_add(top),
            )
                .into(),
            (
                (area.size.w - left - right).max(1),
                (area.size.h - top - bottom).max(1),
            )
                .into(),
        )
    }

    /// A frame always reserves one positive client pixel on each axis.
    pub(crate) fn border_width(&self, frame: Rectangle<i32, Logical>) -> i32 {
        self.border
            .width
            .min((frame.size.w - 1).max(0) / 2)
            .min((frame.size.h - 1).max(0) / 2)
    }

    pub(crate) fn client_rect(&self, frame: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
        let width = self.border_width(frame);
        Rectangle::new(
            frame.loc + Point::from((width, width)),
            (frame.size.w - 2 * width, frame.size.h - 2 * width).into(),
        )
    }
}
