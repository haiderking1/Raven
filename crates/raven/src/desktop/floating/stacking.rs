use crate::state::State;
use smithay::desktop::Window;

impl State {
    /// Ignore the whole parent edge when it leads into a cycle.
    pub(crate) fn valid_floating_parent(&self, window: &Window) -> Option<&Window> {
        let parent = self.toplevel_parent(window)?;
        let mut current = parent;
        for _ in 0..self.windows.len() {
            if current == window {
                return None;
            }
            match self.toplevel_parent(current) {
                Some(next) => current = next,
                None => return Some(parent),
            }
        }
        None
    }

    /// Repair only at lifecycle/focus boundaries, never in input or render traversal.
    pub(crate) fn restack_floating(&mut self, index: usize) {
        let workspace = &self.workspaces.entries[index];
        let mut pending: Vec<_> = workspace
            .space
            .elements()
            .filter(|window| {
                workspace.floating.entries.contains_key(window)
                    || self.valid_floating_parent(window).is_some()
            })
            .cloned()
            .collect();
        let mut ordered = Vec::with_capacity(pending.len());
        while !pending.is_empty() {
            let next = pending
                .iter()
                .position(|window| {
                    let mut current = self.valid_floating_parent(window);
                    while let Some(parent) = current {
                        if pending.contains(parent) {
                            return false;
                        }
                        current = self.toplevel_parent(parent);
                    }
                    true
                })
                .unwrap_or(0);
            ordered.push(pending.remove(next));
        }
        let workspace = &mut self.workspaces.entries[index];
        for window in &ordered {
            workspace.space.raise_element(window, false);
        }
        workspace.floating.elevated = ordered.iter().cloned().collect();
        workspace.floating.stack = ordered;
    }
}
