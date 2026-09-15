use crate::{
    desktop::tests::{
        fixture::{Fixture, Toplevel},
        wire::{Event, string, word},
    },
    state::State,
};
use smithay::{
    desktop::Window,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Resource},
};
use std::time::Duration;
pub(super) struct Harness {
    pub f: Fixture,
    event_loop: EventLoop<'static, State>,
    tops: Vec<Toplevel>,
    pub manager: u32,
    pub events: Vec<Event>,
}
impl Harness {
    pub fn new(version: u32) -> Self {
        let mut f = Fixture::new();
        let event_loop = EventLoop::try_new().unwrap();
        f.state.loop_signal = event_loop.get_signal();
        f.state
            .install_resize_transactions(event_loop.handle())
            .unwrap();
        let output = Output::new(
            "management-test".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "test".into(),
                model: "test".into(),
            },
        );
        output.change_current_state(
            Some(Mode {
                size: (1000, 700).into(),
                refresh: 60000,
            }),
            None,
            None,
            Some((0, 0).into()),
        );
        output.create_global::<State>(&f.state.display_handle);
        f.state.space_mut().map_output(&output, (0, 0));
        f.state.output = Some(output);
        let registry = f.id();
        f.wire.request(1, 1, &[registry]);
        let globals = f.dispatch();
        let global = globals
            .iter()
            .find(|e| {
                e.object == registry && e.opcode == 0 && {
                    let n = word(&e.args[4..]) as usize;
                    &e.args[8..8 + n - 1] == b"zwlr_foreign_toplevel_manager_v1"
                }
            })
            .unwrap();
        let manager = f.id();
        let mut args = word(&global.args).to_ne_bytes().to_vec();
        args.extend(string("zwlr_foreign_toplevel_manager_v1"));
        args.extend(version.to_ne_bytes());
        args.extend(manager.to_ne_bytes());
        f.wire.bytes(registry, 0, &args, None);
        f.dispatch();
        Self {
            f,
            event_loop,
            tops: Vec::new(),
            manager,
            events: Vec::new(),
        }
    }
    pub fn settle(&mut self) {
        for _ in 0..16 {
            let events = self.f.dispatch();
            for top in self.tops.clone() {
                if let Some(configure) = events
                    .iter()
                    .rev()
                    .find(|e| e.object == top.xdg && e.opcode == 0)
                {
                    let size = events
                        .iter()
                        .rev()
                        .find(|e| e.object == top.role && e.opcode == 0)
                        .map(|e| (word(&e.args), word(&e.args[4..])))
                        .unwrap_or((100, 100));
                    self.f.wire.request(top.xdg, 4, &[word(&configure.args)]);
                    let buffer = self.f.buffer_sized(size.0.max(100), size.1.max(100));
                    self.f.wire.request(top.surface, 1, &[buffer, 0, 0]);
                    self.f.wire.request(top.surface, 6, &[]);
                }
            }
            self.events.extend(events);
            self.event_loop
                .dispatch(Duration::ZERO, &mut self.f.state)
                .unwrap();
            self.f.state.refresh();
            self.f.state.refresh_app_switcher();
        }
    }
    pub fn app(&mut self, name: &str) -> (Toplevel, Window, u32) {
        self.app_options(name, None, false)
    }
    pub fn app_options(
        &mut self,
        name: &str,
        parent: Option<Toplevel>,
        maximized: bool,
    ) -> (Toplevel, Window, u32) {
        let top = self.f.toplevel();
        if let Some(parent) = parent {
            self.f.wire.request(top.role, 1, &[parent.role]);
        }
        if maximized {
            self.f.wire.request(top.role, 9, &[]);
        }
        self.tops.push(top);
        self.f.wire.bytes(top.role, 3, &string(name), None);
        self.f.wire.bytes(top.role, 2, &string(name), None);
        self.f.wire.request(top.surface, 6, &[]);
        self.settle();
        let window = self
            .f
            .state
            .windows
            .iter()
            .find(|w| w.toplevel().unwrap().wl_surface().id().protocol_id() == top.surface)
            .unwrap()
            .clone();
        let handle = self
            .events
            .iter()
            .rev()
            .find(|e| {
                e.opcode == 1 && {
                    let n = word(&e.args) as usize;
                    e.args.get(4..4 + n.saturating_sub(1)) == Some(name.as_bytes())
                }
            })
            .unwrap()
            .object;
        (top, window, handle)
    }
    pub fn seat(&mut self) -> u32 {
        let registry = self.f.id();
        self.f.wire.request(1, 1, &[registry]);
        let events = self.f.dispatch();
        let global = events
            .iter()
            .find(|e| {
                e.object == registry && e.opcode == 0 && {
                    let n = word(&e.args[4..]) as usize;
                    &e.args[8..8 + n - 1] == b"wl_seat"
                }
            })
            .unwrap();
        let seat = self.f.id();
        let mut args = word(&global.args).to_ne_bytes().to_vec();
        args.extend(string("wl_seat"));
        args.extend(1u32.to_ne_bytes());
        args.extend(seat.to_ne_bytes());
        self.f.wire.bytes(registry, 0, &args, None);
        self.f.dispatch();
        seat
    }
    pub fn request(&mut self, handle: u32, opcode: u16) {
        self.events.clear();
        self.f.wire.request(handle, opcode, &[]);
        self.settle();
    }
    pub fn states(&self, handle: u32) -> Vec<u32> {
        self.events
            .iter()
            .rev()
            .find(|e| e.object == handle && e.opcode == 4)
            .map(|e| e.args[4..].chunks_exact(4).map(word).collect())
            .unwrap_or_default()
    }
}
