use crate::state::State;
use smithay::reexports::{
    wayland_protocols_misc::server_decoration::server::org_kde_kwin_server_decoration::{
        Mode, OrgKdeKwinServerDecoration, Request,
    },
    wayland_server::{Client, DataInit, Dispatch, DisplayHandle, protocol::wl_surface::WlSurface},
};
use std::sync::Mutex;

/// Resource-owned state disappears on release or client disconnect.
pub(super) struct Decoration {
    _surface: WlSurface,
    replies: Mutex<u8>,
}

impl Decoration {
    pub(super) fn new(surface: WlSurface) -> Self {
        Self {
            _surface: surface,
            replies: Mutex::new(0),
        }
    }
}

impl Dispatch<OrgKdeKwinServerDecoration, Decoration> for State {
    fn request(
        _state: &mut Self,
        _client: &Client,
        resource: &OrgKdeKwinServerDecoration,
        request: Request,
        data: &Decoration,
        _display: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            Request::RequestMode { .. } => {
                // Match Hyprland: answer at most four requests per object.
                // A client can echo mode events, so unbounded replies can loop.
                let mut replies = data.replies.lock().unwrap();
                if *replies < 4 {
                    *replies += 1;
                    resource.mode(Mode::Server);
                }
            }
            Request::Release => {}
            _ => unreachable!(),
        }
    }
}
