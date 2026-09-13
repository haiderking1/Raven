use super::virtual_pointer::{Event, Handler, PointerData, VirtualPointerState};
use crate::state::State;
use smithay::{
    input::Seat,
    output::Output,
    reexports::{
        wayland_protocols_wlr::virtual_pointer::v1::server::{
            zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
            zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
        },
        wayland_server::{
            Resource,
            backend::ObjectId,
            protocol::{wl_output::WlOutput, wl_seat::WlSeat},
        },
    },
};

pub(crate) struct Mapping {
    seat: Option<Seat<State>>,
    output: Option<Output>,
    valid_output: bool,
}
impl Handler for State {
    type Mapping = Mapping;
    fn map_pointer(&mut self, seat: Option<WlSeat>, output: Option<WlOutput>) -> Mapping {
        let seat = match seat {
            Some(seat) => Seat::from_resource(&seat),
            None => Some(self.seat.clone()),
        };
        let mapped_output = output.as_ref().and_then(Output::from_resource);
        Mapping {
            seat,
            valid_output: output.is_none() || mapped_output.is_some(),
            output: mapped_output,
        }
    }
    fn pointer_active(&self, mapping: &Mapping) -> bool {
        mapping.seat.as_ref() == Some(&self.seat)
            && mapping.valid_output
            && mapping
                .output
                .as_ref()
                .is_none_or(|output| self.space().output_geometry(output).is_some())
            && self
                .backend
                .as_ref()
                .is_some_and(|backend| backend.input_active())
    }
    fn pointer_epoch(&self) -> u64 {
        self.input.virtual_pointer_epoch
    }
    fn pointer_event(&mut self, pointer: &ZwlrVirtualPointerV1, mapping: &Mapping, event: Event) {
        self.virtual_pointer_event(pointer.id(), mapping.output.as_ref(), event);
    }
    fn pointer_destroyed(&mut self, id: ObjectId) {
        self.remove_virtual_pointer(id);
    }
}

smithay::reexports::wayland_server::delegate_global_dispatch!(State: [ZwlrVirtualPointerManagerV1: ()] => VirtualPointerState);
smithay::reexports::wayland_server::delegate_dispatch!(State: [ZwlrVirtualPointerManagerV1: ()] => VirtualPointerState);
smithay::reexports::wayland_server::delegate_dispatch!(State: [ZwlrVirtualPointerV1: PointerData<State>] => VirtualPointerState);
