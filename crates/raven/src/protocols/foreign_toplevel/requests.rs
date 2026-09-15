use super::{Handle, HandleData};
use crate::state::State;
use smithay::reexports::{
    wayland_protocols_wlr::foreign_toplevel::v1::server::zwlr_foreign_toplevel_handle_v1::{
        Error, Request,
    },
    wayland_server::{Client, DataInit, Dispatch, DisplayHandle, Resource},
};
use std::sync::atomic::Ordering;
impl Dispatch<Handle, HandleData> for State {
    fn request(
        state: &mut Self,
        _: &Client,
        resource: &Handle,
        request: Request,
        data: &HandleData,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        if !data.active.load(Ordering::Relaxed) {
            return;
        }
        let window = &data.window;
        match request {
            Request::SetMaximized => {
                state.set_window_maximized(window, true);
            }
            Request::UnsetMaximized => {
                state.set_window_maximized(window, false);
            }
            Request::SetMinimized => {
                state.cancel_management_request_for(window);
                state.set_window_minimized(window, true);
            }
            Request::UnsetMinimized => {
                state.restore_managed_window(window);
            }
            Request::Activate { seat } => {
                if smithay::input::Seat::<State>::from_resource(&seat).as_ref() == Some(&state.seat)
                {
                    state.request_taskbar_action(window, false, resource);
                }
            }
            Request::Close => {
                state.cancel_management_request_for(window);
                if state.restore_managed_window(window) {
                    if let Some(top) = window.toplevel() {
                        top.send_close();
                    }
                }
            }
            Request::SetFullscreen { .. } => {
                state.request_taskbar_action(window, true, resource);
            }
            Request::UnsetFullscreen => {
                state.cancel_management_request_for(window);
                if let Some(top) = window.toplevel() {
                    state.request_fullscreen(top, false);
                }
            }
            Request::SetRectangle { width, height, .. } => {
                if width < 0 || height < 0 {
                    resource.post_error(Error::InvalidRectangle, "negative rectangle dimensions");
                }
            }
            Request::Destroy => {}
            _ => unreachable!(),
        }
        state.refresh_foreign_toplevels();
    }
}
