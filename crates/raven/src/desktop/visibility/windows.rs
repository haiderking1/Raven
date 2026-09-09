use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{IsAlive, Logical, Point},
};

impl State {
    pub(crate) fn window_is_visible(&self, window: &Window) -> bool {
        window.alive()
            && self.space().element_location(window).is_some()
            && self.fullscreen_window().is_none_or(|owner| {
                owner == window || self.fullscreen_transient_depth(window).is_some()
            })
    }

    /// Bottom to top, with parents below descendants regardless of focus raises.
    pub(crate) fn visible_windows(&self) -> impl DoubleEndedIterator<Item = &Window> {
        let fullscreen = self.fullscreen_window().is_some();
        let mut windows = Vec::new();
        if fullscreen {
            windows.extend(
                self.space()
                    .elements()
                    .filter(|window| self.window_is_visible(window)),
            );
            // Stable sorting retains Space order among siblings at the same depth.
            windows.sort_by_key(|window| self.fullscreen_transient_depth(window).unwrap_or(0));
        }
        // The ordinary floating stack is repaired on lifecycle/focus changes.
        // Both layout tiers stay borrowed on pointer motion and normal rendering.
        let floating = &self.workspaces.entries[self.workspaces.active].floating;
        self.space()
            .elements()
            .filter(move |window| {
                !fullscreen && !floating.elevated.contains(window) && self.window_is_visible(window)
            })
            .chain(
                floating
                    .stack
                    .iter()
                    .filter(move |window| !fullscreen && self.window_is_visible(window)),
            )
            .chain(windows)
    }

    /// Space places the window geometry, not the root buffer's origin.
    pub(crate) fn window_surface_origin(&self, window: &Window) -> Option<Point<i32, Logical>> {
        Some(self.space().element_location(window)? - window.geometry().loc)
    }
}
