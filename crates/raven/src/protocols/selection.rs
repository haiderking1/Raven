use crate::state::State;
use smithay::{
    delegate_data_device,
    input::Seat,
    reexports::wayland_server::protocol::{wl_data_source::WlDataSource, wl_surface::WlSurface},
    wayland::selection::{
        SelectionHandler,
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
        },
    },
};
use std::os::fd::OwnedFd;

impl SelectionHandler for State {
    type SelectionUserData = Option<std::sync::Arc<[u8]>>;
    fn send_selection(
        &mut self,
        ty: smithay::wayland::selection::SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
        _seat: Seat<Self>,
        user_data: &Self::SelectionUserData,
    ) {
        if ty == smithay::wayland::selection::SelectionTarget::Clipboard && mime_type == "image/png"
        {
            if let Some(png) = user_data {
                self.send_screenshot_clipboard(fd, png.clone());
            }
        }
    }
}
impl DataDeviceHandler for State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}
impl ClientDndGrabHandler for State {
    fn started(
        &mut self,
        _source: Option<WlDataSource>,
        icon: Option<WlSurface>,
        _seat: Seat<Self>,
    ) {
        self.cancel_workspace_activation();
        self.dnd_icon = icon;
        self.request_redraw();
    }
    fn dropped(&mut self, _target: Option<WlSurface>, _validated: bool, _seat: Seat<Self>) {
        self.dnd_icon = None;
        self.request_redraw();
    }
}
impl ServerDndGrabHandler for State {
    // Raven does not start server drags. Clipboard image transfers are handled above.
    // Closing an unexpected transfer FD reports EOF rather than leaving it hanging.
    fn send(&mut self, _mime_type: String, fd: OwnedFd, _seat: Seat<Self>) {
        drop(fd);
    }
}
delegate_data_device!(State);
