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
    type SelectionUserData = ();
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
        self.dnd_icon = icon;
    }
    fn dropped(&mut self, _target: Option<WlSurface>, _validated: bool, _seat: Seat<Self>) {
        self.dnd_icon = None;
    }
}
impl ServerDndGrabHandler for State {
    // Raven never creates a server-owned selection or starts a server drag.
    // Closing an unexpected transfer FD reports EOF rather than leaving it hanging.
    fn send(&mut self, _mime_type: String, fd: OwnedFd, _seat: Seat<Self>) {
        drop(fd);
    }
}
delegate_data_device!(State);
