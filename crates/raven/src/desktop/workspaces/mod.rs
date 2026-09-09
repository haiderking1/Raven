mod access;
mod focus;
mod grabs;
mod movement;
mod switching;
mod visibility;

use super::{floating::Floating, fullscreen::Fullscreen, tiling::Tiling};
use smithay::{
    desktop::{Space, Window},
    utils::IsAlive,
};
use std::collections::HashMap;

pub(crate) const COUNT: usize = 10;

#[derive(Default)]
pub(crate) struct Workspace {
    pub(crate) space: Space<Window>,
    pub(crate) tiling: Tiling,
    pub(crate) floating: Floating,
    pub(crate) fullscreen: Fullscreen,
    pub(crate) focused: Option<Window>,
}

pub(crate) struct Workspaces {
    pub(crate) active: usize,
    pub(crate) entries: [Workspace; COUNT],
    // Ownership survives null-buffer unmaps and delayed first buffers.
    owners: HashMap<Window, usize>,
}

impl Default for Workspaces {
    fn default() -> Self {
        Self {
            active: 0,
            entries: std::array::from_fn(|_| Workspace::default()),
            owners: HashMap::new(),
        }
    }
}

impl Workspaces {
    pub(crate) fn index_of(&self, window: &Window) -> Option<usize> {
        self.owners.get(window).copied()
    }

    pub(crate) fn assign(&mut self, window: Window, index: usize) {
        self.owners.insert(window, index);
    }

    pub(crate) fn forget(&mut self, window: &Window) {
        self.owners.remove(window);
    }

    pub(crate) fn refresh(&mut self) {
        self.owners.retain(|window, _| window.alive());
        for workspace in &mut self.entries {
            workspace.space.refresh();
            if workspace
                .focused
                .as_ref()
                .is_some_and(|window| workspace.space.element_location(window).is_none())
            {
                workspace.focused = workspace.space.elements().next_back().cloned();
            }
        }
    }
}
