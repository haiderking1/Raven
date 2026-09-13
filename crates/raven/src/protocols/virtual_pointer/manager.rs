use super::{Handler, PointerData, VirtualPointerState};
use smithay::reexports::{
    wayland_protocols_wlr::virtual_pointer::v1::server::{
        zwlr_virtual_pointer_manager_v1::{Request, ZwlrVirtualPointerManagerV1},
        zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
    },
    wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New},
};

impl<D> GlobalDispatch<ZwlrVirtualPointerManagerV1, (), D> for VirtualPointerState
where
    D: Handler
        + GlobalDispatch<ZwlrVirtualPointerManagerV1, ()>
        + Dispatch<ZwlrVirtualPointerManagerV1, ()>
        + Dispatch<ZwlrVirtualPointerV1, PointerData<D>>,
{
    fn bind(
        _: &mut D,
        _: &DisplayHandle,
        _: &Client,
        resource: New<ZwlrVirtualPointerManagerV1>,
        _: &(),
        init: &mut DataInit<'_, D>,
    ) {
        init.init(resource, ());
    }
}

impl<D> Dispatch<ZwlrVirtualPointerManagerV1, (), D> for VirtualPointerState
where
    D: Handler
        + Dispatch<ZwlrVirtualPointerManagerV1, ()>
        + Dispatch<ZwlrVirtualPointerV1, PointerData<D>>,
{
    fn request(
        state: &mut D,
        _: &Client,
        _: &ZwlrVirtualPointerManagerV1,
        request: Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, D>,
    ) {
        let (id, seat, output) = match request {
            Request::CreateVirtualPointer { id, seat } => (id, seat, None),
            Request::CreateVirtualPointerWithOutput { id, seat, output } => (id, seat, output),
            Request::Destroy => return,
            _ => unreachable!(),
        };
        let mapping = state.map_pointer(seat, output);
        init.init(
            id,
            PointerData::<D> {
                mapping,
                pending: Default::default(),
            },
        );
    }
}
