use smithay::utils::{Logical, Point, Rectangle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Layout {
    pub rect: Rectangle<i32, Logical>,
    pub first: usize,
    pub visible: usize,
    pub slot: i32,
    pub icon: i32,
}
impl Layout {
    pub fn new(area: Rectangle<i32, Logical>, count: usize, selected: usize) -> Option<Self> {
        if count == 0 || area.size.w < 120 || area.size.h < 120 {
            return None;
        }
        let available = (area.size.w - 48).min(1320);
        let slot = ((available - 48) / count.min(i32::MAX as usize) as i32).clamp(64, 120);
        let visible = ((available - 48) / slot).max(1) as usize;
        let visible = visible.min(count);
        let first = selected.saturating_sub(visible / 2).min(count - visible);
        let icon = (slot - 28).min((area.size.h - 96).max(24));
        let size: smithay::utils::Size<i32, Logical> =
            (visible as i32 * slot + 48, icon + 80).into();
        let rect = Rectangle::new(
            (
                area.loc.x + (area.size.w - size.w) / 2,
                area.loc.y + (area.size.h - size.h) / 2,
            )
                .into(),
            size,
        );
        Some(Self {
            rect,
            first,
            visible,
            slot,
            icon,
        })
    }
    pub fn hit(&self, point: Point<f64, Logical>) -> Option<usize> {
        if !self.rect.to_f64().contains(point) {
            return None;
        }
        let y = point.y - f64::from(self.rect.loc.y);
        if y < 24.0 || y >= f64::from(self.rect.size.h - 12) {
            return None;
        }
        let x = point.x - f64::from(self.rect.loc.x + 24);
        if x < 0.0 || x >= f64::from(self.slot) * self.visible as f64 {
            return None;
        }
        Some(self.first + (x / f64::from(self.slot)) as usize)
    }
}
