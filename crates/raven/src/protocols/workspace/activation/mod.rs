//! A panel may request activation before its implicit click grab has ended.
mod click;
mod lifecycle;

use super::{ExtWorkspaceHandleV1, ExtWorkspaceManagerV1};
use crate::state::State;
use click::{Click, Status};
use smithay::reexports::wayland_server::{Resource, Weak};

pub(super) struct Deferred {
    manager: Weak<ExtWorkspaceManagerV1>,
    workspace: Weak<ExtWorkspaceHandleV1>,
    source: usize,
    target: usize,
    click: Click,
}

impl State {
    pub(super) fn activate_workspace_from_protocol(
        &mut self,
        manager: &ExtWorkspaceManagerV1,
        target: usize,
    ) {
        // One committed intent globally, including a request for the current workspace.
        self.cancel_workspace_activation();
        if target == self.workspaces.active {
            return;
        }
        if let Some(click) = Click::held(self) {
            let workspace = self
                .workspace_protocol
                .subscriptions
                .iter()
                .find(|subscription| subscription.manager.id() == manager.id())
                .and_then(|subscription| subscription.workspaces.get(target))
                .and_then(Option::as_ref)
                .cloned();
            if let Some(workspace) = workspace {
                self.workspace_protocol.deferred = Some(Deferred {
                    manager: manager.downgrade(),
                    workspace,
                    source: self.workspaces.active,
                    target,
                    click,
                });
            }
            return;
        }
        // Window selection, DnD and other grabs retain the existing rejection policy.
        self.switch_workspace(target);
    }

    pub(super) fn refresh_workspace_activation(&mut self) {
        let Some(pending) = self.workspace_protocol.deferred.take() else {
            return;
        };
        if pending.source != self.workspaces.active
            || pending.manager.upgrade().is_err()
            || pending.workspace.upgrade().is_err()
        {
            return;
        }
        match pending.click.status(self) {
            Status::Held => self.workspace_protocol.deferred = Some(pending),
            // Smithay has already delivered the release to the original surface.
            // Now focus and fullscreen layer visibility can safely change together.
            Status::Released => self.switch_workspace(pending.target),
            Status::Cancelled => {}
        }
    }
}
