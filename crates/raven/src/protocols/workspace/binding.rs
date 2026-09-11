use super::{Subscription, WorkspaceData};
use crate::{desktop::workspaces::COUNT, state::State};
use smithay::reexports::{
    wayland_protocols::ext::workspace::v1::server::{
        ext_workspace_group_handle_v1::{ExtWorkspaceGroupHandleV1, GroupCapabilities},
        ext_workspace_handle_v1::{ExtWorkspaceHandleV1, State as Flags, WorkspaceCapabilities},
        ext_workspace_manager_v1::ExtWorkspaceManagerV1,
    },
    wayland_server::{Client, DataInit, DisplayHandle, GlobalDispatch, New, Resource},
};

impl GlobalDispatch<ExtWorkspaceManagerV1, ()> for State {
    fn bind(
        state: &mut Self,
        display: &DisplayHandle,
        client: &Client,
        resource: New<ExtWorkspaceManagerV1>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        let manager = init.init(resource, ());
        let group = client
            .create_resource::<ExtWorkspaceGroupHandleV1, _, Self>(display, 1, manager.id())
            .expect("live manager client");
        manager.workspace_group(&group);
        group.capabilities(GroupCapabilities::empty());
        let outputs: Vec<_> = state
            .output
            .as_ref()
            .map(|output| {
                output
                    .client_outputs(client)
                    .filter(Resource::is_alive)
                    .collect()
            })
            .unwrap_or_default();
        for output in &outputs {
            group.output_enter(output);
        }
        let active = state.workspaces.active;
        let workspaces = std::array::from_fn::<_, COUNT, _>(|index| {
            let workspace = client
                .create_resource::<ExtWorkspaceHandleV1, _, Self>(
                    display,
                    1,
                    WorkspaceData {
                        manager: manager.id(),
                        index,
                    },
                )
                .expect("live manager client");
            manager.workspace(&workspace);
            workspace.id(format!("raven-workspace-{}", index + 1));
            workspace.name((index + 1).to_string());
            workspace.coordinates((index as u32).to_ne_bytes().to_vec());
            workspace.state(if index == active {
                Flags::Active
            } else {
                Flags::empty()
            });
            workspace.capabilities(WorkspaceCapabilities::Activate);
            group.workspace_enter(&workspace);
            Some(workspace.downgrade())
        });
        manager.done();
        state.workspace_protocol.subscriptions.push(Subscription {
            manager: manager.downgrade(),
            group: Some(group.downgrade()),
            workspaces,
            outputs: outputs.iter().map(Resource::downgrade).collect(),
            active,
            pending: None,
        });
    }
}
