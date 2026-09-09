use super::super::{
    fixture::Fixture,
    wire::{Event, string, word},
};
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::server::{
    zwlr_layer_shell_v1::Layer,
    zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
};

#[derive(Clone, Copy)]
pub(in crate::desktop::tests) struct LayerClient {
    pub surface: u32,
    pub role: u32,
}

impl LayerClient {
    pub fn new(f: &mut Fixture, shell: u32) -> Self {
        let layer = Self {
            surface: f.id(),
            role: f.id(),
        };
        f.wire.request(3, 0, &[layer.surface]);
        // Null output exercises the default-output path used by Fuzzel.
        let mut args: Vec<_> = [layer.role, layer.surface, 0, Layer::Top as u32]
            .into_iter()
            .flat_map(u32::to_ne_bytes)
            .collect();
        args.extend(string("fuzzel"));
        f.wire.bytes(shell, 0, &args, None);
        layer
    }

    pub fn settings(
        self,
        f: &mut Fixture,
        anchor: Anchor,
        zone: i32,
        keyboard: KeyboardInteractivity,
    ) {
        f.wire.request(self.role, 0, &[100, 100]);
        f.wire.request(self.role, 1, &[anchor.bits()]);
        f.wire.request(self.role, 2, &[zone as u32]);
        self.keyboard(f, keyboard);
    }

    pub fn keyboard(self, f: &mut Fixture, keyboard: KeyboardInteractivity) {
        f.wire.request(self.role, 4, &[keyboard as u32]);
    }

    pub fn commit(self, f: &mut Fixture) -> Vec<Event> {
        f.wire.request(self.surface, 6, &[]);
        f.dispatch()
    }

    pub fn configure(self, f: &mut Fixture) -> u32 {
        let events = self.commit(f);
        let configs: Vec<_> = events
            .iter()
            .filter(|event| event.object == self.role && event.opcode == 0)
            .collect();
        assert_eq!(configs.len(), 1, "one initial configure per mapping cycle");
        let event = configs[0];
        assert_eq!((word(&event.args[4..]), word(&event.args[8..])), (100, 100));
        word(&event.args)
    }

    pub fn ack(self, f: &mut Fixture, serial: u32) {
        f.wire.request(self.role, 6, &[serial]);
        f.dispatch();
    }

    pub fn attach(self, f: &mut Fixture, buffer: u32) -> Vec<Event> {
        f.wire.request(self.surface, 1, &[buffer, 0, 0]);
        self.commit(f)
    }

    pub fn frame(self, f: &mut Fixture) -> u32 {
        let callback = f.id();
        f.wire.request(self.surface, 3, &[callback]);
        self.commit(f);
        callback
    }
}
