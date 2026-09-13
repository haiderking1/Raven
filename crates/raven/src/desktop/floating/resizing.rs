use super::{configure::bounded_size, hints::Hints};
use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle},
};

impl State {
    /// Client geometry is asynchronous feedback, not a new preferred size while
    /// the pointer owns this allocation. After release, only a response to the
    /// final target may choose a different size (for example a terminal grid).
    pub(super) fn floating_resize_owns_size(&self, window: &Window) -> bool {
        if !self.window_has_live_resize(window) {
            return false;
        }
        let Some(top) = window.toplevel() else {
            return false;
        };
        self.surface_is_dragged_window(top.wl_surface())
            || self
                .window_target_client_geometry(window)
                .is_some_and(|target| top.current_state().size != Some(target.size))
    }

    pub(crate) fn resize_floating_window(
        &mut self,
        window: &Window,
        initial: Rectangle<i32, Logical>,
        delta: Point<i32, Logical>,
        left: bool,
        top: bool,
    ) {
        if self.fullscreen_manages(window) {
            return;
        }
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let Some(area) = self.tiling_area() else {
            return;
        };
        let border = self.appearance.border_width(area);
        let requested = (
            (initial.size.w - 2 * border + if left { -delta.x } else { delta.x }).max(1),
            (initial.size.h - 2 * border + if top { -delta.y } else { delta.y }).max(1),
        )
            .into();
        let size = bounded_size(&Hints::committed(window), Some(requested), None);
        let frame_size = (size.w + 2 * border, size.h + 2 * border);
        let position = initial.loc
            + Point::from((
                if left {
                    initial.size.w - frame_size.0
                } else {
                    0
                },
                if top {
                    initial.size.h - frame_size.1
                } else {
                    0
                },
            ));
        let Some(entry) = self.workspaces.entries[index].floating.entries.get(window) else {
            return;
        };
        if entry.natural == Some(size) && entry.position == Some(position) {
            return;
        }
        self.begin_resize_batch(index);
        let entry = self.workspaces.entries[index]
            .floating
            .entries
            .get_mut(window)
            .unwrap();
        entry.natural = Some(size);
        entry.position = Some(position);
        entry.manual_size = true;
        self.arrange_floating(index);
        self.end_resize_batch();
        self.request_redraw();
    }
}
