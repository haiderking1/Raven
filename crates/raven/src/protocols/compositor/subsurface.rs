use crate::state::State;
use smithay::{
    reexports::wayland_server::{
        Client, DataInit, Dispatch, DisplayHandle,
        backend::ClientId,
        protocol::wl_subsurface::{self, WlSubsurface},
    },
    wayland::compositor::{CompositorState, SubsurfaceUserData},
};

impl Dispatch<WlSubsurface, SubsurfaceUserData> for State {
    fn request(
        state: &mut Self,
        client: &Client,
        object: &WlSubsurface,
        request: wl_subsurface::Request,
        data: &SubsurfaceUserData,
        display: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        <CompositorState as Dispatch<WlSubsurface, SubsurfaceUserData, State>>::request(
            state, client, object, request, data, display, init,
        );
    }

    fn destroyed(
        state: &mut Self,
        client: ClientId,
        object: &WlSubsurface,
        data: &SubsurfaceUserData,
    ) {
        <CompositorState as Dispatch<WlSubsurface, SubsurfaceUserData, State>>::destroyed(
            state, client, object, data,
        );
        // Destroying only the role unparents the surface immediately. There may
        // be no later wl_surface.commit or wl_surface.destroy to request a frame.
        state.subsurface_role_removed();
    }
}
