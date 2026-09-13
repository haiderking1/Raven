//! Legacy decoration advertisement used by GTK and Ghostty.
mod surface;

use crate::state::State;
use smithay::reexports::{
    wayland_protocols_misc::server_decoration::server::{
        org_kde_kwin_server_decoration::Mode as SurfaceMode,
        org_kde_kwin_server_decoration_manager::{
            Mode, OrgKdeKwinServerDecorationManager, Request,
        },
    },
    wayland_server::{
        Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, backend::GlobalId,
    },
};
use surface::Decoration;

pub(crate) struct KdeDecorations {
    _global: GlobalId,
}

impl KdeDecorations {
    pub(crate) fn new(display: &DisplayHandle) -> Self {
        Self {
            _global: display.create_global::<State, OrgKdeKwinServerDecorationManager, _>(1, ()),
        }
    }
}

impl GlobalDispatch<OrgKdeKwinServerDecorationManager, ()> for State {
    fn bind(
        _state: &mut Self,
        _display: &DisplayHandle,
        _client: &Client,
        resource: New<OrgKdeKwinServerDecorationManager>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, ()).default_mode(Mode::Server);
    }
}

impl Dispatch<OrgKdeKwinServerDecorationManager, ()> for State {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &OrgKdeKwinServerDecorationManager,
        request: Request,
        _data: &(),
        _display: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            Request::Create { id, surface } => {
                data_init
                    .init(id, Decoration::new(surface))
                    .mode(SurfaceMode::Server);
            }
            _ => unreachable!(),
        }
    }
}
