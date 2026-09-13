use crate::{
    protocol::{Event, Handler, PointerData, VirtualPointerState},
    wire::{Wire, string, word},
};
use smithay::{
    input::{Seat, SeatState},
    output::{Output, PhysicalProperties, Subpixel},
    reexports::{
        wayland_protocols_wlr::virtual_pointer::v1::server::{
            zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
            zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
        },
        wayland_server::{
            Display, Resource,
            backend::ObjectId,
            protocol::{wl_output::WlOutput, wl_seat::WlSeat},
        },
    },
    wayland::compositor::CompositorState,
};
use std::{os::unix::net::UnixStream, sync::Arc};

pub struct State {
    pub seats: SeatState<Self>,
    pub compositor: CompositorState,
    _seat: Seat<Self>,
    pub active: bool,
    pub epoch: u64,
    pub events: Vec<Event>,
    pub mappings: Vec<(Option<u32>, Option<u32>)>,
    pub destroyed: Vec<ObjectId>,
}
impl Handler for State {
    type Mapping = (Option<u32>, Option<u32>);
    fn map_pointer(&mut self, seat: Option<WlSeat>, output: Option<WlOutput>) -> Self::Mapping {
        let mapping = (
            seat.map(|s| s.id().protocol_id()),
            output.map(|o| o.id().protocol_id()),
        );
        self.mappings.push(mapping);
        mapping
    }
    fn pointer_active(&self, _: &Self::Mapping) -> bool {
        self.active
    }
    fn pointer_epoch(&self) -> u64 {
        self.epoch
    }
    fn pointer_event(&mut self, _: &ZwlrVirtualPointerV1, _: &Self::Mapping, event: Event) {
        self.events.push(event);
    }
    fn pointer_destroyed(&mut self, id: ObjectId) {
        self.destroyed.push(id);
    }
}
smithay::reexports::wayland_server::delegate_global_dispatch!(State: [ZwlrVirtualPointerManagerV1: ()] => VirtualPointerState);
smithay::reexports::wayland_server::delegate_dispatch!(State: [ZwlrVirtualPointerManagerV1: ()] => VirtualPointerState);
smithay::reexports::wayland_server::delegate_dispatch!(State: [ZwlrVirtualPointerV1: PointerData<State>] => VirtualPointerState);

pub struct Server {
    pub display: Display<State>,
    pub state: State,
    pub wire: Wire,
}
impl Server {
    pub fn new(version: u32) -> Self {
        let display = Display::new().unwrap();
        let mut handle = display.handle();
        let compositor = CompositorState::new::<State>(&handle);
        let mut seats = SeatState::new();
        let mut seat = seats.new_wl_seat(&handle, "test-seat");
        seat.add_pointer();
        let output = Output::new(
            "test-output".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "test".into(),
                model: "test".into(),
            },
        );
        output.create_global::<State>(&handle);
        VirtualPointerState::new::<State>(&handle);
        let state = State {
            seats,
            compositor,
            _seat: seat,
            active: true,
            epoch: 0,
            events: Vec::new(),
            mappings: Vec::new(),
            destroyed: Vec::new(),
        };
        let (server, socket) = UnixStream::pair().unwrap();
        handle
            .insert_client(server, Arc::new(crate::globals::Client::default()))
            .unwrap();
        let mut server = Self {
            display,
            state,
            wire: Wire::new(socket),
        };
        server.wire.request(1, 1, &[2]);
        server.pump();
        let globals = server.wire.events();
        for (name, id, bind_version) in [
            ("zwlr_virtual_pointer_manager_v1", 3, version),
            ("wl_seat", 4, 1),
            ("wl_output", 5, 1),
        ] {
            let global = globals
                .iter()
                .find(|e| {
                    e.object == 2 && e.opcode == 0 && {
                        let len = word(&e.args[4..]) as usize;
                        &e.args[8..8 + len - 1] == name.as_bytes()
                    }
                })
                .expect("required global");
            let mut args = word(&global.args).to_ne_bytes().to_vec();
            args.extend(string(name));
            args.extend(bind_version.to_ne_bytes());
            args.extend(u32::to_ne_bytes(id));
            server.wire.bytes(2, 0, &args, None);
        }
        server.pump();
        server.wire.events();
        server
    }
    pub fn pump(&mut self) {
        self.display.dispatch_clients(&mut self.state).unwrap();
        self.display.flush_clients().unwrap();
    }
    pub fn create(&mut self) {
        self.wire.request(3, 0, &[0, 6]);
        self.pump();
    }
}
