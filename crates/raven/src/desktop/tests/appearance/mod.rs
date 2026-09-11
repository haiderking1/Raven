mod fixture;
use super::{
    layers::{fixture::fixture, protocol::LayerClient},
    workspaces::fixture::window,
};
use crate::desktop::appearance::{Appearance, InnerGaps, OuterGaps};
use fixture::{ack, allocation, configured};
use smithay::{
    output::Mode,
    reexports::wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1::{
        Anchor, KeyboardInteractivity,
    },
};

#[test]
fn appearance_configures_frames_input_and_restores_floating_size() {
    let (mut f, shell) = fixture();
    f.state.set_appearance(Appearance::default()).unwrap();
    let oversized = f.buffer_sized(800, 600);
    let first = f.toplevel();
    f.wire.request(first.surface, 6, &[]);
    let events = f.dispatch();
    let serial = configured(&events, first, (780, 580));
    ack(&mut f, first, serial);
    f.attach(first, oversized);
    let master = window(&f, first);
    allocation(&f, &master, (8, 8, 784, 584), 2);

    let second = f.toplevel();
    f.configure(second);
    f.attach(second, oversized);
    let third = f.toplevel();
    f.wire.request(third.surface, 6, &[]);
    let events = f.dispatch();
    let serial = configured(&events, third, (384, 284));
    ack(&mut f, third, serial);
    f.attach(third, oversized);
    let upper = window(&f, second);
    let lower = window(&f, third);
    allocation(&f, &master, (8, 8, 388, 584), 2);
    allocation(&f, &upper, (404, 8, 388, 288), 2);
    allocation(&f, &lower, (404, 304, 388, 288), 2);
    // Oversized buffers cannot receive pointer input on either kind of gap.
    for point in [(4.0, 40.0), (400.0, 40.0), (450.0, 300.0)] {
        assert!(f.state.surface_under(point.into()).is_none());
    }
    f.state.focus_window_at((8.0, 40.0).into());
    assert_eq!(f.state.focused_window(), Some(master.clone()));
    assert!(f.state.surface_under((8.0, 40.0).into()).is_none());
    assert!(f.state.surface_under((10.0, 40.0).into()).is_some());
    let region = f.id();
    f.wire.request(3, 1, &[region]);
    f.wire.request(first.surface, 5, &[region]);
    f.wire.request(first.surface, 6, &[]);
    f.dispatch();
    assert!(
        f.state.surface_under((10.0, 40.0).into()).is_none(),
        "client input region still applies"
    );
    f.state.focus_window_at((8.0, 40.0).into());
    assert_eq!(f.state.focused_window(), Some(master.clone()));
    f.wire.request(first.surface, 5, &[0]);
    f.wire.request(region, 0, &[]);
    f.wire.request(first.surface, 6, &[]);
    f.dispatch();
    // Focus raises must never change the persistent tile order.
    allocation(&f, &master, (8, 8, 388, 584), 2);
    f.dispatch();

    let custom = Appearance {
        inner: InnerGaps {
            horizontal: 20,
            vertical: 12,
        },
        outer: OuterGaps {
            top: 12,
            right: 16,
            bottom: 20,
            left: 24,
        },
        border: crate::desktop::appearance::Border {
            width: 3,
            active: [0.8, 0.4, 0.2, 0.5],
            ..Appearance::default().border
        },
    };
    f.state.set_appearance(custom).unwrap();
    let events = f.dispatch();
    configured(&events, first, (364, 562));
    configured(&events, second, (364, 272));
    allocation(&f, &master, (24, 12, 370, 568), 3);
    allocation(&f, &upper, (414, 12, 370, 278), 3);
    allocation(&f, &lower, (414, 302, 370, 278), 3);
    assert_eq!(custom.border.premultiplied(true), [0.4, 0.2, 0.1, 0.5]);
    f.state.take_redraw_request();
    for invalid in [
        Appearance {
            inner: InnerGaps {
                horizontal: -1,
                ..custom.inner
            },
            ..custom
        },
        Appearance {
            outer: OuterGaps {
                left: i32::MAX,
                ..custom.outer
            },
            ..custom
        },
        Appearance {
            border: crate::desktop::appearance::Border {
                active: [f32::NAN; 4],
                ..custom.border
            },
            ..custom
        },
    ] {
        assert!(f.state.set_appearance(invalid).is_err());
        assert_eq!(f.state.appearance, custom);
        assert!(f.dispatch().is_empty());
        assert!(!f.state.take_redraw_request());
    }
    allocation(&f, &master, (24, 12, 370, 568), 3);
    f.state.set_appearance(custom).unwrap();
    assert!(f.dispatch().is_empty());
    assert!(!f.state.take_redraw_request());
    f.state.set_appearance(Appearance::disabled()).unwrap();
    let events = f.dispatch();
    configured(&events, first, (400, 600));
    allocation(&f, &master, (0, 0, 400, 600), 0);
    allocation(&f, &upper, (400, 0, 400, 300), 0);

    // A real exclusive layer changes the workarea before outer gaps are applied.
    f.state.set_appearance(Appearance::default()).unwrap();
    f.dispatch();
    let panel = LayerClient::new(&mut f, shell);
    panel.settings(&mut f, Anchor::Top, 40, KeyboardInteractivity::None);
    let serial = panel.configure(&mut f);
    panel.ack(&mut f, serial);
    let panel_buffer = f.buffer();
    let events = panel.attach(&mut f, panel_buffer);
    configured(&events, first, (384, 540));
    allocation(&f, &master, (8, 48, 388, 544), 2);

    f.wire.request(first.role, 11, &[0]);
    let events = f.dispatch();
    let serial = configured(&events, first, (800, 600));
    ack(&mut f, first, serial);
    f.wire.request(first.surface, 6, &[]);
    f.dispatch();
    assert_eq!(f.state.fullscreen_window(), Some(&master));
    allocation(&f, &master, (0, 0, 800, 600), 0);

    let dialog = f.toplevel();
    f.wire.request(dialog.role, 1, &[first.role]);
    f.configure(dialog);
    let natural = f.buffer_sized(300, 200);
    f.attach(dialog, natural);
    let floating = window(&f, dialog);
    assert!(f.state.window_is_floating(&floating));
    assert_eq!(
        f.state.window_client_geometry(&floating).unwrap().size,
        (300, 200).into()
    );
    assert_eq!(
        f.state.window_frame_geometry(&floating).unwrap().size,
        (304, 204).into()
    );
    assert!(f.state.window_frame_geometry(&floating).unwrap().loc.y >= 48);
    // Fullscreen owner stays gapless while its related dialog keeps its frame.
    f.state.set_appearance(custom).unwrap();
    let events = f.dispatch();
    assert!(!events.iter().any(|e| e.object == first.role));
    allocation(&f, &master, (0, 0, 800, 600), 0);
    assert_eq!(
        f.state.window_frame_geometry(&floating).unwrap().size,
        (306, 206).into()
    );

    let huge = Appearance {
        inner: InnerGaps {
            horizontal: 65_535,
            vertical: 65_535,
        },
        outer: OuterGaps {
            top: 65_535,
            right: 65_535,
            bottom: 65_535,
            left: 65_535,
        },
        border: crate::desktop::appearance::Border {
            width: 65_535,
            ..custom.border
        },
    };
    f.state.set_appearance(huge).unwrap();
    let events = f.dispatch();
    let clamp_serial = configured(&events, dialog, (1, 1));
    configured(&events, second, (1, 1));
    allocation(&f, &master, (0, 0, 800, 600), 0);
    let clamp = f.buffer_sized(1, 1);
    // Relax before the old configure arrives. Its commit must not erase natural size.
    f.state.set_appearance(Appearance::default()).unwrap();
    let events = f.dispatch();
    let restored = configured(&events, dialog, (300, 200));
    ack(&mut f, dialog, clamp_serial);
    f.attach(dialog, clamp);
    assert_eq!(
        f.state.floating_geometry(&floating).unwrap().size,
        (300, 200).into()
    );
    ack(&mut f, dialog, restored);
    f.attach(dialog, natural);

    // Tiny output plus excessive gaps cannot produce a zero configure or border.
    let output = f.state.output.as_ref().unwrap().clone();
    output.change_current_state(
        Some(Mode {
            size: (2, 1).into(),
            refresh: 60_000,
        }),
        None,
        None,
        None,
    );
    f.state.refresh();
    f.state.set_appearance(huge).unwrap();
    let events = f.dispatch();
    configured(&events, dialog, (1, 1));
    let client = f.state.window_client_geometry(&floating).unwrap();
    assert_eq!(client.size, (1, 1).into());
    assert_eq!(f.state.window_frame_geometry(&floating), Some(client));
    assert_eq!(f.state.fullscreen_window(), Some(&master));
    assert_eq!(
        f.state.window_frame_geometry(&master),
        f.state.window_client_geometry(&master)
    );
}
