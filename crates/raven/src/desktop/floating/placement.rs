use super::{configure::bounded_size, hints::Hints};
use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle},
};

impl State {
    pub(super) fn compute_floating_geometry(
        &self,
        window: &Window,
    ) -> Option<(Rectangle<i32, Logical>, Rectangle<i32, Logical>)> {
        let area = self
            .tiling_area()
            .filter(|a| a.size.w > 0 && a.size.h > 0)?;
        let index = self.workspaces.index_of(window)?;
        let entry = self.workspaces.entries[index]
            .floating
            .entries
            .get(window)?;
        let bounds = self.appearance.client_rect(area).size;
        let border = self.appearance.border_width(area);
        let mut size = bounded_size(&Hints::committed(window), entry.natural, Some(bounds));
        // Placement needs an extent even before the client chooses an unconstrained axis.
        if size.w == 0 {
            size.w = (bounds.w / 2).max(1);
        }
        if size.h == 0 {
            size.h = (bounds.h / 2).max(1);
        }
        let client_size = size;
        size = (size.w + 2 * border, size.h + 2 * border).into();
        let parent = self
            .valid_floating_parent(window)
            .filter(|parent| self.workspaces.index_of(parent) == Some(index))
            .filter(|parent| {
                self.workspaces.entries[index]
                    .space
                    .element_location(parent)
                    .is_some()
            });
        let anchor = parent
            .and_then(|parent| self.window_target_client_geometry(parent))
            .unwrap_or(area);
        let centered =
            anchor.loc + Point::from(((anchor.size.w - size.w) / 2, (anchor.size.h - size.h) / 2));
        let loc = Point::from((
            centered
                .x
                .clamp(area.loc.x, area.loc.x + area.size.w - size.w),
            centered
                .y
                .clamp(area.loc.y, area.loc.y + area.size.h - size.h),
        ));
        Some((
            Rectangle::new(loc, size),
            Rectangle::new(loc + Point::from((border, border)), client_size),
        ))
    }

    pub(crate) fn arrange_floating(&mut self, index: usize) {
        if self.defer_resize_layout(index) {
            return;
        }
        let floating = &self.workspaces.entries[index].floating;
        if floating.entries.is_empty() && floating.stack.is_empty() {
            return;
        }
        self.begin_resize_batch(index);
        self.restack_floating(index);
        let windows = self.workspaces.entries[index].floating.stack.clone();
        let mut changed = false;
        for window in windows.iter().cloned() {
            if !self.window_is_floating(&window) {
                continue;
            }
            let allocation = self.compute_floating_geometry(&window);
            let frame = allocation.map(|(frame, _)| frame);
            let geometry = allocation.map(|(_, client)| client);
            let entry = self.workspaces.entries[index]
                .floating
                .entries
                .get_mut(&window)
                .unwrap();
            changed |= entry.geometry != geometry || entry.frame != frame;
            entry.geometry = geometry;
            entry.frame = frame;
            if self.fullscreen_manages(&window) {
                self.configure_fullscreen(&window, false);
            } else {
                self.configure_floating(&window);
                self.send_resize_configure(&window);
            }
            if let Some(loc) = self
                .fullscreen_location(&window)
                .or_else(|| geometry.map(|g| g.loc))
            {
                changed |= self.workspaces.entries[index]
                    .space
                    .element_location(&window)
                    != Some(loc);
                self.place_resize_window(index, &window, loc);
            }
        }
        // map_element raises changed windows. Restore the complete prior tier,
        // including unchanged siblings and mapped tiled descendants.
        let space = &mut self.workspaces.entries[index].space;
        for window in &windows {
            space.raise_element(window, false);
        }
        self.end_resize_batch();
        if changed {
            self.refresh_reactive_popups(index);
            if index == self.workspaces.active {
                self.request_redraw();
                self.refresh_tiling_pointer();
            }
        }
    }

    /// Report placement changes, not ordinary pixel/frame-callback commits.
    pub(crate) fn commit_floating_size(&mut self, window: &Window) -> bool {
        if self.fullscreen_manages(window) {
            return false;
        }
        let Some(index) = self.workspaces.index_of(window) else {
            return false;
        };
        if !self.workspaces.entries[index]
            .floating
            .entries
            .contains_key(window)
        {
            return false;
        }
        let hints = Hints::committed(window);
        let entry = self.workspaces.entries[index]
            .floating
            .entries
            .get_mut(window)
            .unwrap();
        let mut changed = entry.hints.as_ref() != Some(&hints);
        entry.hints = Some(hints);
        let size = window.geometry().size;
        // The committed XDG state follows the acknowledged configure, even if
        // a newer gap/workarea configure is already pending. A late response to
        // an older clamp must not replace the natural size after bounds relax.
        let configured = window.toplevel().and_then(|top| top.current_state().size);
        if size.w > 0
            && size.h > 0
            && (entry.natural.is_none()
                || (configured != Some(size)
                    && entry.geometry.is_none_or(|area| area.size != size)))
            && entry.natural != Some(size)
        {
            // Complying with a workarea clamp must not erase the preferred size.
            entry.natural = Some(size);
            changed = true;
        }
        changed
    }
}
