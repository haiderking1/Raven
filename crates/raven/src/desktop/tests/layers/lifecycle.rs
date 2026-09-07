use super::{assertions::*, fixture::*, protocol::LayerClient};
use smithay::{
    desktop::layer_map_for_output,
    reexports::wayland_protocols_wlr::layer_shell::v1::server::{
        zwlr_layer_shell_v1::Layer as WireLayer,
        zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
    },
    utils::Rectangle,
    wayland::shell::wlr_layer::{Layer, LayerSurfaceCachedState},
};
use std::time::Duration;

#[test]
fn only_mapped_layers_reserve_tiles_and_null_buffer_starts_a_fresh_role_cycle() {
    let (mut f, shell) = fixture();
    let buffer = f.buffer();
    let (top, window) = map_window(&mut f, buffer);
    let panel = LayerClient::new(&mut f, shell);
    panel.settings(&mut f, Anchor::Top, 40, KeyboardInteractivity::Exclusive);
    f.wire.request(panel.role, 8, &[WireLayer::Overlay as u32]);
    f.wire.request(panel.role, 3, &[5, 0, 0, 0]);
    let serial = panel.configure(&mut f);
    let pending_callback = panel.frame(&mut f);
    panel.ack(&mut f, serial);
    f.state.refresh();
    mapped(&f, panel, false);
    focus(&f, top.surface);
    assert!(f.state.surface_under((360.0, 10.0).into()).is_none());
    assert_eq!(
        f.state.space().element_location(&window),
        Some((0, 0).into())
    );
    assert_eq!(
        layer_map_for_output(f.state.output.as_ref().unwrap()).non_exclusive_zone(),
        Rectangle::new((0, 0).into(), (800, 600).into())
    );
    f.state.send_frames(Duration::from_millis(10));
    assert!(!frame_done(&f.dispatch(), pending_callback));

    let events = panel.attach(&mut f, buffer);
    mapped(&f, panel, true);
    focus(&f, panel.surface);
    hit(&f, (360.0, 10.0), panel.surface);
    tile_size(&events, top, (800, 555));
    assert_eq!(
        f.state.space().element_location(&window),
        Some((0, 45).into())
    );
    assert_eq!(
        layer_map_for_output(f.state.output.as_ref().unwrap()).non_exclusive_zone(),
        Rectangle::new((0, 45).into(), (800, 555).into())
    );
    f.state.send_frames(Duration::from_millis(20));
    assert!(frame_done(&f.dispatch(), pending_callback));

    let events = panel.attach(&mut f, 0);
    mapped(&f, panel, false);
    focus(&f, top.surface);
    tile_size(&events, top, (800, 600));
    assert_eq!(
        f.state.space().element_location(&window),
        Some((0, 0).into())
    );
    assert!(f.state.surface_under((360.0, 10.0).into()).is_none());
    let reset = layer(&f, panel).cached_state();
    let defaults = LayerSurfaceCachedState::default();
    assert_eq!(reset.size, defaults.size);
    assert_eq!(reset.anchor, defaults.anchor);
    assert_eq!(reset.exclusive_zone, defaults.exclusive_zone);
    assert_eq!(
        (
            reset.margin.top,
            reset.margin.right,
            reset.margin.bottom,
            reset.margin.left
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        reset.keyboard_interactivity,
        defaults.keyboard_interactivity
    );
    assert_eq!(reset.layer, Layer::Top, "unmap restores the creation layer");

    // Only size is set again. Neither pending nor current role state may leak.
    f.wire.request(panel.role, 0, &[100, 100]);
    let remap_serial = panel.configure(&mut f);
    assert_ne!(serial, remap_serial);
    let hidden_callback = panel.frame(&mut f);
    panel.ack(&mut f, remap_serial);
    f.state.send_frames(Duration::from_millis(30));
    assert!(!frame_done(&f.dispatch(), hidden_callback));
    assert!(f.state.surface_under((360.0, 260.0).into()).is_none());
    mapped(&f, panel, false);
    focus(&f, top.surface);
    assert_eq!(
        layer_map_for_output(f.state.output.as_ref().unwrap()).non_exclusive_zone(),
        Rectangle::new((0, 0).into(), (800, 600).into())
    );
    panel.attach(&mut f, buffer);
    mapped(&f, panel, true);
    focus(&f, top.surface);
    hit(&f, (360.0, 260.0), panel.surface);
    let output = f.state.output.as_ref().unwrap();
    assert_eq!(
        layer_map_for_output(output).layer_geometry(&layer(&f, panel)),
        Some(Rectangle::new((350, 250).into(), (100, 100).into()))
    );
    assert_eq!(
        layer_map_for_output(output).non_exclusive_zone(),
        Rectangle::new((0, 0).into(), (800, 600).into())
    );
    f.state.send_frames(Duration::from_millis(40));
    assert!(frame_done(&f.dispatch(), hidden_callback));
}
