use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle, Size},
};

impl State {
    /// Legacy tiled transients use a centered allocation while
    /// their mapped ancestor owns fullscreen. Never borrow a displaced entry's area.
    pub(crate) fn fullscreen_transient_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        self.fullscreen_transient_depth(window)?;
        if self.window_is_floating(window) {
            return self.floating_geometry(window);
        }
        let area = self.fullscreen_area()?;
        let index = self.workspaces.index_of(window)?;
        let mapped = self.workspaces.entries[index]
            .space
            .element_location(window)
            .is_some();
        let requested = if mapped {
            window.geometry().size
        } else {
            // Before a buffer exists, offer a useful dialog size rather than a tile.
            Size::from((area.size.w / 2, area.size.h / 2))
        };
        let size = Size::from((
            requested.w.clamp(1, area.size.w),
            requested.h.clamp(1, area.size.h),
        ));
        let loc = area.loc + Point::from(((area.size.w - size.w) / 2, (area.size.h - size.h) / 2));
        Some(Rectangle::new(loc, size))
    }
}
