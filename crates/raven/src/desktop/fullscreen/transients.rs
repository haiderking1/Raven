use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle, Size},
};

use std::sync::Mutex;

#[derive(Default)]
struct TransientFrame(Mutex<Option<Rectangle<i32, Logical>>>);

impl State {
    /// Legacy tiled transients use a centered allocation while
    /// their mapped ancestor owns fullscreen. Never borrow a displaced entry's area.
    pub(crate) fn fullscreen_transient_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        self.fullscreen_transient_frame_geometry(window)
            .map(|frame| self.appearance.client_rect(frame))
    }

    pub(crate) fn fullscreen_transient_frame_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        self.fullscreen_transient_depth(window)?;
        if self.window_is_floating(window) {
            return self.floating_frame_geometry(window);
        }
        *window
            .user_data()
            .get::<TransientFrame>()?
            .0
            .lock()
            .unwrap()
    }

    /// Called during configure/placement, never during rendering or hit testing.
    pub(crate) fn refresh_fullscreen_transient_geometry(
        &self,
        window: &Window,
    ) -> Option<Rectangle<i32, Logical>> {
        let frame = self.compute_transient_frame(window)?;
        window
            .user_data()
            .insert_if_missing(TransientFrame::default);
        *window
            .user_data()
            .get::<TransientFrame>()
            .unwrap()
            .0
            .lock()
            .unwrap() = Some(frame);
        Some(self.appearance.client_rect(frame))
    }

    fn compute_transient_frame(&self, window: &Window) -> Option<Rectangle<i32, Logical>> {
        self.fullscreen_transient_depth(window)?;
        let area = self.tiling_area()?;
        let bounds = self.appearance.client_rect(area).size;
        let border = self.appearance.border_width(area);
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
            requested.w.clamp(1, bounds.w) + 2 * border,
            requested.h.clamp(1, bounds.h) + 2 * border,
        ));
        let loc = area.loc + Point::from(((area.size.w - size.w) / 2, (area.size.h - size.h) / 2));
        Some(Rectangle::new(loc, size))
    }
}
