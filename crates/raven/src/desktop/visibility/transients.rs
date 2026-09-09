use crate::state::State;
use smithay::{desktop::Window, utils::IsAlive};

impl State {
    pub(crate) fn toplevel_parent(&self, window: &Window) -> Option<&Window> {
        let parent = window.toplevel()?.parent()?;
        self.windows.iter().find(|candidate| {
            candidate.alive()
                && candidate
                    .toplevel()
                    .is_some_and(|t| t.wl_surface() == &parent)
        })
    }

    /// The child may be awaiting its first buffer; every ancestor must be mapped.
    pub(crate) fn fullscreen_transient_depth(&self, window: &Window) -> Option<usize> {
        let index = self.workspaces.index_of(window)?;
        let workspace = &self.workspaces.entries[index];
        let owner = workspace.fullscreen.displayed.as_ref()?;
        self.fullscreen_area()?;
        if owner == window || !window.alive() {
            return None;
        }
        // A cycle through the owner is not a valid transient relationship either.
        self.valid_floating_parent(window)?;
        let mut current = window;
        // A valid chain cannot contain more ancestors than live windows.
        // Bounded traversal handles malformed cycles without allocating a set.
        for depth in 1..=self.windows.len() {
            let parent = self.toplevel_parent(current)?;
            if self.workspaces.index_of(parent) != Some(index)
                || workspace.space.element_location(parent).is_none()
            {
                return None;
            }
            if parent == owner {
                return Some(depth);
            }
            current = parent;
        }
        None
    }
}
