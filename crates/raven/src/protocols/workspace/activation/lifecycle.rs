use crate::state::State;
use smithay::reexports::wayland_server::{
    Resource, backend::ObjectId, protocol::wl_surface::WlSurface,
};

impl State {
    /// Explicit workspace intent, VT suspension and new grabs supersede a panel click.
    pub(crate) fn cancel_workspace_activation(&mut self) {
        self.workspace_protocol.deferred = None;
    }

    pub(in crate::protocols::workspace) fn cancel_workspace_activation_manager(
        &mut self,
        id: &ObjectId,
    ) {
        if self
            .workspace_protocol
            .deferred
            .as_ref()
            .is_some_and(|pending| pending.manager.id() == *id)
        {
            self.cancel_workspace_activation();
        }
    }

    pub(in crate::protocols::workspace) fn cancel_workspace_activation_handle(
        &mut self,
        id: &ObjectId,
    ) {
        if self
            .workspace_protocol
            .deferred
            .as_ref()
            .is_some_and(|pending| pending.workspace.id() == *id)
        {
            self.cancel_workspace_activation();
        }
    }

    pub(crate) fn cancel_workspace_activation_surface(&mut self, surface: &WlSurface) {
        if self
            .workspace_protocol
            .deferred
            .as_ref()
            .is_some_and(|pending| pending.click.root.id() == surface.id())
        {
            self.cancel_workspace_activation();
        }
    }
}
