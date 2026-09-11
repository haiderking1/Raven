//! ext-workspace-v1 for Raven's fixed, output-spanning workspace set.
mod activation;
mod binding;
mod dispatch;
mod refresh;
#[cfg(test)]
mod tests;

use crate::desktop::workspaces::COUNT;
use smithay::reexports::{
    wayland_protocols::ext::workspace::v1::server::{
        ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1,
        ext_workspace_handle_v1::ExtWorkspaceHandleV1,
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
    last_active: usize,
    last_output: Option<smithay::output::Output>,
    outputs_dirty: bool,
    deferred: Option<activation::Deferred>,
}

struct Subscription {
    manager: Weak<ExtWorkspaceManagerV1>,
    group: Option<Weak<ExtWorkspaceGroupHandleV1>>,
    workspaces: [Option<Weak<ExtWorkspaceHandleV1>>; COUNT],
    outputs: Vec<Weak<WlOutput>>,
    active: usize,
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
            last_active: usize::MAX,
            last_output: None,
            outputs_dirty: true,
            deferred: None,
        }
    }
}
