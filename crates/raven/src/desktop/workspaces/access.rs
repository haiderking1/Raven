use crate::state::State;
use smithay::desktop::{Space, Window};

impl State {
    pub(crate) fn space(&self) -> &Space<Window> {
        &self.workspaces.entries[self.workspaces.active].space
    }

    pub(crate) fn space_mut(&mut self) -> &mut Space<Window> {
        &mut self.workspaces.entries[self.workspaces.active].space
    }
}
