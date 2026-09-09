use super::{
    super::{
        fixture::{Fixture, Toplevel},
        wire::{Event, word},
    },
    protocol::LayerClient,
};
use smithay::{
    desktop::{LayerSurface, layer_map_for_output},
    reexports::wayland_server::Resource,
};

pub(in crate::desktop::tests) fn layer(f: &Fixture, client: LayerClient) -> LayerSurface {
    f.state
        .layers
        .surfaces
        .iter()
        .find(|layer| layer.wl_surface().id().protocol_id() == client.surface)
        .expect("layer role is registered")
        .clone()
}

pub(in crate::desktop::tests) fn focus(f: &Fixture, surface: u32) {
    assert_eq!(
        f.state
            .seat
            .get_keyboard()
            .unwrap()
            .current_focus()
            .map(|surface| surface.id().protocol_id()),
        Some(surface)
    );
}

pub(in crate::desktop::tests) fn hit(f: &Fixture, point: (f64, f64), surface: u32) {
    assert_eq!(
        f.state
            .surface_under(point.into())
            .map(|(surface, _)| surface.id().protocol_id()),
        Some(surface)
    );
}

pub(in crate::desktop::tests) fn mapped(f: &Fixture, client: LayerClient, expected: bool) {
    let map = layer_map_for_output(f.state.output.as_ref().unwrap());
    assert_eq!(map.layer_geometry(&layer(f, client)).is_some(), expected);
}

pub(in crate::desktop::tests) fn frame_done(events: &[Event], callback: u32) -> bool {
    events
        .iter()
        .any(|event| event.object == callback && event.opcode == 0)
}

pub(in crate::desktop::tests) fn tile_size(events: &[Event], top: Toplevel, expected: (u32, u32)) {
    let configure = events
        .iter()
        .rev()
        .find(|event| event.object == top.role && event.opcode == 0)
        .expect("tiling sends a new window size");
    assert_eq!(
        (word(&configure.args), word(&configure.args[4..])),
        expected
    );
}
