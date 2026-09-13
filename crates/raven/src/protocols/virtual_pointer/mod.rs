//! wlr-virtual-pointer-unstable-v1, versions 1 and 2.
mod dispatch;
mod frame;
mod manager;

use smithay::{
    input::pointer::AxisFrame,
    reexports::{
        wayland_protocols_wlr::virtual_pointer::v1::server::{
            zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
            zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
        },
        wayland_server::{
            Dispatch, DisplayHandle, GlobalDispatch,
            backend::{GlobalId, ObjectId},
            protocol::{wl_output::WlOutput, wl_seat::WlSeat},
        },
    },
};
use std::sync::Mutex;

#[derive(Debug)]
pub(crate) enum Event {
    Motion {
        time: u32,
        dx: f64,
        dy: f64,
    },
    Absolute {
        time: u32,
        x: u32,
        y: u32,
        x_extent: u32,
        y_extent: u32,
    },
    Button {
        time: u32,
        button: u32,
        pressed: bool,
    },
    Frame(Option<AxisFrame>),
}

pub(crate) trait Handler: Sized + 'static {
    type Mapping: Send + Sync + 'static;
    fn map_pointer(&mut self, seat: Option<WlSeat>, output: Option<WlOutput>) -> Self::Mapping;
    fn pointer_active(&self, mapping: &Self::Mapping) -> bool;
    fn pointer_epoch(&self) -> u64;
    fn pointer_event(
        &mut self,
        pointer: &ZwlrVirtualPointerV1,
        mapping: &Self::Mapping,
        event: Event,
    );
    fn pointer_destroyed(&mut self, id: ObjectId);
}

pub(crate) struct PointerData<D: Handler> {
    mapping: D::Mapping,
    pending: Mutex<frame::Pending>,
}

pub(crate) struct VirtualPointerState {
    _global: GlobalId,
}
impl VirtualPointerState {
    pub(crate) fn new<D>(display: &DisplayHandle) -> Self
    where
        D: Handler
            + GlobalDispatch<ZwlrVirtualPointerManagerV1, ()>
            + Dispatch<ZwlrVirtualPointerManagerV1, ()>
            + Dispatch<ZwlrVirtualPointerV1, PointerData<D>>,
    {
        Self {
            _global: display.create_global::<D, ZwlrVirtualPointerManagerV1, _>(2, ()),
        }
    }
}
