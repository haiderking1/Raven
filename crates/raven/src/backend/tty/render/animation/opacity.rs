use smithay::utils::{Physical, Rectangle};

type Rect = Rectangle<i32, Physical>;

/// Transform declared opaque coverage through the same client projection as
/// content, but round inward and leave room for bilinear edge filtering.
pub(super) fn project(regions: Vec<Rect>, source: Rect, target: Rect, element: Rect) -> Vec<Rect> {
    let x = f64::from(target.size.w) / f64::from(source.size.w);
    let y = f64::from(target.size.h) / f64::from(source.size.h);
    let inset_x = if x == 1.0 { 0.0 } else { x.max(1.0) };
    let inset_y = if y == 1.0 { 0.0 } else { y.max(1.0) };
    regions
        .into_iter()
        .filter_map(|region| {
            let lo = (
                target.loc.x + (f64::from(region.loc.x - source.loc.x) * x + inset_x).ceil() as i32,
                target.loc.y + (f64::from(region.loc.y - source.loc.y) * y + inset_y).ceil() as i32,
            );
            let hi = (
                target.loc.x
                    + (f64::from(region.loc.x + region.size.w - source.loc.x) * x - inset_x).floor()
                        as i32,
                target.loc.y
                    + (f64::from(region.loc.y + region.size.h - source.loc.y) * y - inset_y).floor()
                        as i32,
            );
            if hi.0 <= lo.0 || hi.1 <= lo.1 {
                return None;
            }
            let mut region = Rectangle::from_extremities(lo, hi).intersection(element)?;
            region.loc -= element.loc;
            Some(region)
        })
        .collect()
}
