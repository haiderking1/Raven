mod config;
pub(super) use config::persistent_from_env;

use crate::{desktop::workspaces::COUNT, state::State};
use smithay::{
    reexports::wayland_protocols::ext::workspace::v1::server::ext_workspace_handle_v1::State as Flags,
    utils::IsAlive,
};

impl State {
    /// Hidden is a panel hint, not workspace removal or window visibility.
    /// Count mapped windows on every workspace, including those covered by fullscreen.
    pub(super) fn workspace_visibility_states(&self) -> [Flags; COUNT] {
        std::array::from_fn(|index| {
            if index == self.workspaces.active {
                Flags::Active
            } else if self.workspace_protocol.persistent[index]
                || self.workspaces.entries[index]
                    .space
                    .elements()
                    .any(|window| window.alive())
            {
                Flags::empty()
            } else {
                Flags::Hidden
            }
        })
    }
}
