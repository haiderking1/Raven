use super::{
    fixture::{Fixture, Toplevel},
    wire::{Event, string, word},
};
use smithay::{input::pointer::RelativeMotionEvent, reexports::wayland_server::Resource};

pub(super) struct Game {
    pub f: Fixture,
    pub top: Toplevel,
    pub neighbor: Toplevel,
    pub pointer: u32,
    pub relative: u32,
    constraints: u32,
}

impl Game {
    pub fn new() -> Self {
        let mut f = Fixture::new();
        let registry = f.id();
        f.wire.request(1, 1, &[registry]);
        let globals = f.dispatch();
        let mut bind = |interface: &str, version: u32| {
            let event = globals
                .iter()
                .find(|event| {
                    event.object == registry && event.opcode == 0 && {
                        let len = word(&event.args[4..]) as usize;
                        &event.args[8..8 + len - 1] == interface.as_bytes()
                    }
                })
                .expect("capture global");
            let id = f.id();
            let mut args = word(&event.args).to_ne_bytes().to_vec();
            args.extend(string(interface));
            args.extend(version.to_ne_bytes());
            args.extend(id.to_ne_bytes());
            f.wire.bytes(registry, 0, &args, None);
            id
        };
        let seat = bind("wl_seat", 5);
        let manager = bind("zwp_relative_pointer_manager_v1", 1);
        let constraints = bind("zwp_pointer_constraints_v1", 1);
        let pointer = f.id();
        let relative = f.id();
        f.wire.request(seat, 0, &[pointer]);
        f.wire.request(manager, 1, &[relative, pointer]);
        f.dispatch();
        let top = f.toplevel();
        f.configure(top);
        let buffer = f.buffer();
        f.attach(top, buffer);
        let neighbor = f.toplevel();
        f.configure(neighbor);
        f.attach(neighbor, buffer);
        let window = f
            .state
            .space()
            .elements()
            .find(|w| w.toplevel().unwrap().wl_surface().id().protocol_id() == neighbor.surface)
            .unwrap()
            .clone();
        f.state.space_mut().map_element(window, (200, 0), false);
        Self {
            f,
            top,
            neighbor,
            pointer,
            relative,
            constraints,
        }
    }

    pub fn region(&mut self, x: u32, width: u32, hole: bool) -> u32 {
        let region = self.f.id();
        self.f.wire.request(3, 1, &[region]);
        self.f.wire.request(region, 1, &[x, 0, width, 100]);
        if hole {
            self.f.wire.request(region, 2, &[40, 0, 10, 100]);
        }
        region
    }

    pub fn constraint(&mut self, locked: bool, region: u32, lifetime: u32) -> u32 {
        let id = self.f.id();
        self.f.wire.request(
            self.constraints,
            if locked { 1 } else { 2 },
            &[id, self.top.surface, self.pointer, region, lifetime],
        );
        id
    }

    pub fn motion(
        &mut self,
        location: (f64, f64),
        relative: Option<RelativeMotionEvent>,
    ) -> Vec<Event> {
        super::super::super::motion(&mut self.f.state, location.into(), 4321, relative);
        self.f.dispatch()
    }

    pub fn commit(&mut self) -> Vec<Event> {
        self.f.wire.request(self.top.surface, 6, &[]);
        self.f.dispatch()
    }
}

pub(super) fn event(events: &[Event], object: u32, opcode: u16) -> &Event {
    let found: Vec<_> = events
        .iter()
        .filter(|event| event.object == object && event.opcode == opcode)
        .collect();
    assert_eq!(found.len(), 1, "expected one {object}:{opcode}: {events:?}");
    found[0]
}

pub(super) fn no_motion(events: &[Event], pointer: u32) {
    assert!(
        !events.iter().any(|e| e.object == pointer && e.opcode <= 2),
        "lock leaked enter/leave/motion: {events:?}"
    );
}
