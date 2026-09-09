use super::super::{assertions::*, fixture::*};
use crate::desktop::tests::layers::{
    assertions::{focus, hit},
    fixture::fixture as layer_fixture,
    protocol::LayerClient,
};
use smithay::reexports::{
    wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1::{
        Anchor, KeyboardInteractivity,
    },
    wayland_server::Resource,
};

#[test]
fn full_output_ignores_reserved_panel_and_exit_restores_three_tile_order() {
    let (mut f, shell) = layer_fixture();
    let buffer = f.buffer();
    let panel = LayerClient::new(&mut f, shell);
    panel.settings(&mut f, Anchor::Top, 40, KeyboardInteractivity::None);
    let serial = panel.configure(&mut f);
    panel.ack(&mut f, serial);
    panel.attach(&mut f, buffer);
    let (_, first) = map(&mut f, buffer);
    let (top, middle) = map(&mut f, buffer);
    let (_, last) = map(&mut f, buffer);
    layout(&f, &first, (0, 40), (400, 560));
    layout(&f, &middle, (400, 40), (400, 280));
    layout(&f, &last, (400, 320), (400, 280));
    let tiles = snapshot(&f, &[&first, &middle, &last]);
    f.state.activate_window(Some(middle.clone()));
    f.dispatch();
    enter(&mut f, top);
    owner(&f, Some(&middle));
    layout(&f, &middle, (0, 0), (800, 600));
    let serial = configured(&request(&mut f, top, false), top, (400, 280), false);
    ack(&mut f, top, serial);
    commit(&mut f, top);
    assert_eq!(snapshot(&f, &[&first, &middle, &last]), tiles);
}

#[test]
fn undersized_geometry_is_centered_and_black_space_does_not_hit_hidden_windows() {
    let mut f = fixture();
    let large = f.buffer_sized(800, 600);
    let (background, hidden) = map(&mut f, large);
    let small = f.buffer();
    let (top, window) = map(&mut f, small);
    f.wire.request(top.xdg, 3, &[10, 12, 80, 70]);
    commit(&mut f, top);
    enter(&mut f, top);
    layout(&f, &window, (0, 0), (800, 600));
    assert_eq!(window.geometry().size, (80, 70).into());
    assert_eq!(
        f.state.space().element_location(&window),
        Some((360, 265).into())
    );
    let (surface, origin) = f.state.surface_under((361.0, 266.0).into()).unwrap();
    assert_eq!(surface.id().protocol_id(), top.surface);
    assert_eq!(origin, (350.0, 253.0).into());
    assert!(!f.state.window_is_visible(&hidden));
    assert!(f.state.surface_under((5.0, 5.0).into()).is_none());
    f.state.focus_window_on_motion((5.0, 5.0).into());
    f.state.focus_window_at((5.0, 5.0).into());
    focus(&f, top.surface);
    owner(&f, Some(&window));
    // The same black-space point really does contain client pixels when tiled.
    let serial = configured(&request(&mut f, top, false), top, (400, 600), false);
    ack(&mut f, top, serial);
    commit(&mut f, top);
    hit(&f, (5.0, 5.0), background.surface);
}
