use super::WorkspaceData;
use crate::state::State;
use smithay::reexports::{
    wayland_protocols::ext::workspace::v1::server::{
        ext_workspace_group_handle_v1::{self, ExtWorkspaceGroupHandleV1},
        ext_workspace_handle_v1::{self, ExtWorkspaceHandleV1},
        ext_workspace_manager_v1::{self, ExtWorkspaceManagerV1},
    },
    wayland_server::{
        Client, DataInit, Dispatch, DisplayHandle, Resource,
        backend::{ClientId, ObjectId},
    },
};

impl Dispatch<ExtWorkspaceManagerV1, ()> for State {
    fn request(
        state: &mut Self,
        _: &Client,
        manager: &ExtWorkspaceManagerV1,
        request: ext_workspace_manager_v1::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        match request {
            ext_workspace_manager_v1::Request::Commit => {
                let Some(subscription) = state
                    .workspace_protocol
                    .subscriptions
                    .iter_mut()
                    .find(|subscription| subscription.manager.id() == manager.id())
                else {
                    return;
                };
                // Raven has one active workspace. Last activate in this manager's
                // transaction wins, without exposing intermediate workspace switches.
                let pending = subscription.pending.take();
                if let Some(index) = pending {
                    state.activate_workspace_from_protocol(manager, index);
                }
                state.publish_workspace_protocol(Some((manager.id(), pending)));
            }
            ext_workspace_manager_v1::Request::Stop => {
                state.cancel_workspace_activation_manager(&manager.id());
                state
                    .workspace_protocol
                    .subscriptions
                    .retain(|subscription| subscription.manager.id() != manager.id());
                // Child handles remain client-owned but inert. Their data holds only IDs.
                manager.finished();
            }
            _ => unreachable!("ext-workspace-v1 manager request"),
        }
    }

    fn destroyed(state: &mut Self, _: ClientId, manager: &ExtWorkspaceManagerV1, _: &()) {
        state.cancel_workspace_activation_manager(&manager.id());
        state
            .workspace_protocol
            .subscriptions
            .retain(|subscription| subscription.manager.id() != manager.id());
    }
}

impl Dispatch<ExtWorkspaceHandleV1, WorkspaceData> for State {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &ExtWorkspaceHandleV1,
        request: ext_workspace_handle_v1::Request,
        data: &WorkspaceData,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        if let ext_workspace_handle_v1::Request::Activate = request {
            if let Some(subscription) = state
                .workspace_protocol
                .subscriptions
                .iter_mut()
                .find(|subscription| subscription.manager.id() == data.manager)
            {
                subscription.pending = Some(data.index);
            }
        }
        // Destroy is handled by destroyed. Unsupported operations are ignored,
        // as required when their capabilities have not been advertised.
    }

    fn destroyed(
        state: &mut Self,
        _: ClientId,
        handle: &ExtWorkspaceHandleV1,
        data: &WorkspaceData,
    ) {
        state.cancel_workspace_activation_handle(&Resource::id(handle));
        if let Some(subscription) = state
            .workspace_protocol
            .subscriptions
            .iter_mut()
            .find(|subscription| subscription.manager.id() == data.manager)
        {
            subscription.workspaces[data.index] = None;
            if subscription.pending == Some(data.index) {
                subscription.pending = None;
            }
        }
    }
}

impl Dispatch<ExtWorkspaceGroupHandleV1, ObjectId> for State {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &ExtWorkspaceGroupHandleV1,
        _: ext_workspace_group_handle_v1::Request,
        _: &ObjectId,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        // Creating workspaces is unsupported. Destroy needs only local bookkeeping.
    }

    fn destroyed(state: &mut Self, _: ClientId, _: &ExtWorkspaceGroupHandleV1, manager: &ObjectId) {
        if let Some(subscription) = state
            .workspace_protocol
            .subscriptions
            .iter_mut()
            .find(|subscription| subscription.manager.id() == *manager)
        {
            subscription.group = None;
            subscription.outputs.clear();
        }
    }
}
