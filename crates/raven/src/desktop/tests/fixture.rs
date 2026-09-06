use super::wire::{Event, Wire, string, word};
use crate::state::{ClientState, State};
use smithay::reexports::{
    calloop::EventLoop,
    rustix::fs::{MemfdFlags, ftruncate, memfd_create},
    wayland_server::Display,
};
use std::{
    os::{fd::AsFd, unix::net::UnixStream},
    sync::Arc,
};

pub struct Fixture {
    pub state: State,
    display: Display<State>,
    pub wire: Wire,
    next_id: u32,
}

#[derive(Clone, Copy)]
pub struct Toplevel {
    pub surface: u32,
    pub xdg: u32,
    pub role: u32,
}

impl Fixture {
    pub fn new() -> Self {
        let display = Display::new().unwrap();
        let event_loop = EventLoop::<State>::try_new().unwrap();
        let state = State::new(display.handle(), event_loop.get_signal()).unwrap();
        let (server, client) = UnixStream::pair().unwrap();
        display
            .handle()
            .insert_client(server, Arc::new(ClientState::default()))
            .unwrap();
        let mut fixture = Self {
            state,
            display,
            wire: Wire::new(client),
            next_id: 6,
        };
        fixture.wire.request(1, 1, &[2]);
        let events = fixture.dispatch();
        for (interface, id, version) in [
            ("wl_compositor", 3, 4),
            ("wl_shm", 4, 1),
            ("xdg_wm_base", 5, 6),
        ] {
            let global = events
                .iter()
                .find(|event| {
                    event.object == 2 && event.opcode == 0 && {
                        let len = word(&event.args[4..]) as usize;
                        &event.args[8..8 + len - 1] == interface.as_bytes()
                    }
                })
                .expect("required global");
            let mut args = word(&global.args).to_ne_bytes().to_vec();
            args.extend(string(interface));
            args.extend(u32::to_ne_bytes(version));
            args.extend(u32::to_ne_bytes(id));
            fixture.wire.bytes(2, 0, &args, None);
        }
        fixture.dispatch();
        fixture
    }

    pub fn id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn dispatch(&mut self) -> Vec<Event> {
        self.display.dispatch_clients(&mut self.state).unwrap();
        self.display.flush_clients().unwrap();
        self.wire.events()
    }

    pub fn toplevel(&mut self) -> Toplevel {
        let top = Toplevel {
            surface: self.id(),
            xdg: self.id(),
            role: self.id(),
        };
        self.wire.request(3, 0, &[top.surface]);
        self.wire.request(5, 2, &[top.xdg, top.surface]);
        self.wire.request(top.xdg, 1, &[top.role]);
        top
    }

    pub fn configure(&mut self, top: Toplevel) {
        self.wire.request(top.surface, 6, &[]);
        let events = self.dispatch();
        let configure = events
            .iter()
            .find(|event| event.object == top.xdg && event.opcode == 0)
            .expect("initial configure");
        self.wire.request(top.xdg, 4, &[word(&configure.args)]);
    }

    pub fn buffer(&mut self) -> u32 {
        let fd = memfd_create(c"raven-protocol-test", MemfdFlags::CLOEXEC).unwrap();
        ftruncate(&fd, 100 * 100 * 4).unwrap();
        let pool = self.id();
        let buffer = self.id();
        let args = [pool, 100 * 100 * 4]
            .iter()
            .flat_map(|v| u32::to_ne_bytes(*v))
            .collect::<Vec<_>>();
        self.wire.bytes(4, 0, &args, Some(fd.as_fd()));
        self.wire.request(pool, 0, &[buffer, 0, 100, 100, 400, 0]);
        self.wire.request(pool, 1, &[]);
        buffer
    }

    pub fn attach(&mut self, top: Toplevel, buffer: u32) {
        self.wire.request(top.surface, 1, &[buffer, 0, 0]);
        self.wire.request(top.surface, 6, &[]);
        self.dispatch();
    }
}
