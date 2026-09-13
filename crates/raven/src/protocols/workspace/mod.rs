//! ext-workspace-v1 for Raven's fixed, output-spanning workspace set.
mod activation;
mod binding;
mod dispatch;
mod refresh;
mod settings;
#[cfg(test)]
mod tests;
mod visibility;

use crate::desktop::workspaces::COUNT;
use smithay::reexports::{
    wayland_protocols::ext::workspace::v1::server::{
        ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1,
        ext_workspace_handle_v1::{ExtWorkspaceHandleV1, State as Flags},
        ext_workspace_manager_v1::ExtWorkspaceManagerV1,
    },
    wayland_server::{
        DisplayHandle, Weak,
        backend::{GlobalId, ObjectId},
        protocol::wl_output::WlOutput,
    },
};

pub(crate) struct WorkspaceProtocol {
    _global: GlobalId,
    subscriptions: Vec<Subscription>,
    last_states: Option<[Flags; COUNT]>,
    persistent: [bool; COUNT],
    show_all: bool,
    last_output: Option<smithay::output::Output>,
    outputs_dirty: bool,
    deferred: Option<activation::Deferred>,
}

struct Subscription {
    manager: Weak<ExtWorkspaceManagerV1>,
    group: Option<Weak<ExtWorkspaceGroupHandleV1>>,
    workspaces: [Option<Weak<ExtWorkspaceHandleV1>>; COUNT],
    outputs: Vec<Weak<WlOutput>>,
    states: [Flags; COUNT],
    pending: Option<usize>,
}

#[derive(Clone)]
pub(crate) struct WorkspaceData {
    manager: ObjectId,
    index: usize,
}

impl WorkspaceProtocol {
    pub(crate) fn outputs_changed(&mut self) {
        self.outputs_dirty = true;
    }

    pub(crate) fn new(display: &DisplayHandle) -> Self {
        Self {
            _global: display.create_global::<crate::state::State, ExtWorkspaceManagerV1, _>(1, ()),
            subscriptions: Vec::new(),
            last_states: None,
            persistent: [false; COUNT],
            show_all: false,
            last_output: None,
            outputs_dirty: true,
            deferred: None,
        }
    }
}
