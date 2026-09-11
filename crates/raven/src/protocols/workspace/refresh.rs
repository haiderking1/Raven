use crate::state::State;
use smithay::reexports::{
    wayland_protocols::ext::workspace::v1::server::ext_workspace_handle_v1::State as Flags,
    wayland_server::{Resource, backend::ObjectId},
};

impl State {
    /// Publish workspace/output changes after dispatch and output synchronization.
    /// Includes wl_output objects bound after the workspace manager. No-op refreshes
    /// emit nothing; each changed manager receives one terminating done event.
    pub fn refresh_workspace_protocol(&mut self) {
        self.refresh_workspace_activation();
        self.publish_workspace_protocol(None);
    }

    pub(super) fn publish_workspace_protocol(
        &mut self,
        acknowledgement: Option<(ObjectId, Option<usize>)>,
    ) {
        let active = self.workspaces.active;
        let output = self.output.as_ref();
        let protocol = &mut self.workspace_protocol;
        if acknowledgement.is_none()
            && !protocol.outputs_dirty
            && protocol.last_active == active
            && protocol.last_output.as_ref() == output
        {
            return;
        }
        protocol.last_active = active;
        protocol.last_output = output.cloned();
        protocol.outputs_dirty = false;
        self.workspace_protocol
            .subscriptions
            .retain_mut(|subscription| {
                let Ok(manager) = subscription.manager.upgrade() else {
                    return false;
                };
                let Some(client) = manager.client() else {
                    return false;
                };
                let acknowledge = acknowledgement
                    .as_ref()
                    .filter(|(id, _)| *id == manager.id());
                let mut changed = false;
                if let Some(group) = subscription
                    .group
                    .as_ref()
                    .and_then(|group| group.upgrade().ok())
                {
                    let outputs: Vec<_> = output
                        .map(|output| {
                            output
                                .client_outputs(&client)
                                .filter(Resource::is_alive)
                                .collect()
                        })
                        .unwrap_or_default();
                    // A released wl_output is not a valid event argument. Forget it without leave.
                    for previous in &subscription.outputs {
                        if let Ok(previous) = previous.upgrade() {
                            if !outputs.contains(&previous) {
                                group.output_leave(&previous);
                                changed = true;
                            }
                        }
                    }
                    for current in &outputs {
                        if !subscription
                            .outputs
                            .iter()
                            .any(|previous| previous == &current.downgrade())
                        {
                            group.output_enter(current);
                            changed = true;
                        }
                    }
                    subscription.outputs = outputs.iter().map(Resource::downgrade).collect();
                } else {
                    subscription.group = None;
                    subscription.outputs.clear();
                }
                for (index, handle) in subscription.workspaces.iter_mut().enumerate() {
                    let Some(workspace) = handle.as_ref().and_then(|handle| handle.upgrade().ok())
                    else {
                        *handle = None;
                        continue;
                    };
                    let state_changed = (index == active) != (index == subscription.active);
                    // Requests may be denied by a grab. Report the requested workspace's
                    // actual state at commit, not an optimistic activation or deferred retry.
                    let requested =
                        acknowledge.is_some_and(|(_, requested)| *requested == Some(index));
                    if state_changed || requested {
                        workspace.state(if index == active {
                            Flags::Active
                        } else {
                            Flags::empty()
                        });
                        changed = true;
                    }
                }
                subscription.active = active;
                if changed || acknowledge.is_some() {
                    manager.done();
                }
                true
            });
    }
}
