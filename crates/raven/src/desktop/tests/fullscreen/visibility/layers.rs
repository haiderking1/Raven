use super::super::{
    assertions::*,
    fixture::{enter, map},
};
use crate::desktop::tests::layers::{assertions::*, fixture::fixture, protocol::LayerClient};
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::server::{
    zwlr_layer_shell_v1::Layer,
    zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
};

#[test]
fn fullscreen_keeps_overlay_and_exclusive_top_launcher_but_suppresses_other_layers() {
    let (mut f, shell) = fixture();
    let buffer = f.buffer_sized(800, 600);
    let (top, window) = map(&mut f, buffer);
    enter(&mut f, top);
    let layer_buffer = f.buffer();
    for (kind, keyboard, visible) in [
        (Layer::Background, KeyboardInteractivity::None, false),
        (Layer::Bottom, KeyboardInteractivity::None, false),
        (Layer::Top, KeyboardInteractivity::OnDemand, false),
        (Layer::Overlay, KeyboardInteractivity::None, true),
        (Layer::Top, KeyboardInteractivity::Exclusive, true),
    ] {
        let client = LayerClient::new(&mut f, shell);
        client.settings(&mut f, Anchor::empty(), 0, keyboard);
        f.wire.request(client.role, 8, &[kind as u32]);
        let serial = client.configure(&mut f);
        client.ack(&mut f, serial);
        assert!(!f.state.layer_is_visible(&layer(&f, client)));
        client.attach(&mut f, layer_buffer);
        assert_eq!(
            f.state.layer_is_visible(&layer(&f, client)),
            visible,
            "{kind:?}/{keyboard:?}"
        );
        hit(
            &f,
            (360.0, 260.0),
            if visible { client.surface } else { top.surface },
        );
        f.state.focus_window_at((360.0, 260.0).into());
        focus(
            &f,
            if keyboard == KeyboardInteractivity::Exclusive {
                client.surface
            } else {
                top.surface
            },
        );
        owner(&f, Some(&window));
        if keyboard == KeyboardInteractivity::Exclusive {
            f.state.toggle_fullscreen();
            assert!(
                !f.dispatch()
                    .iter()
                    .any(|event| event.object == top.role && event.opcode == 0)
            );
            owner(&f, Some(&window));
            focus(&f, client.surface);
            // Keyboard interactivity is double-buffered, just like visibility policy.
            client.keyboard(&mut f, KeyboardInteractivity::None);
            f.dispatch();
            assert!(f.state.layer_is_visible(&layer(&f, client)));
            client.commit(&mut f);
            assert!(!f.state.layer_is_visible(&layer(&f, client)));
            focus(&f, top.surface);
            hit(&f, (360.0, 260.0), top.surface);
        }
        client.attach(&mut f, 0);
        assert!(!f.state.layer_is_visible(&layer(&f, client)));
        focus(&f, top.surface);
    }
}
