use crate::{protocol::Event, server::Server};
use smithay::backend::input::AxisSource;

#[test]
fn versions_accept_motion_clicks_and_output_hints() {
    for version in [1, 2] {
        let mut s = Server::new(version);
        if version == 1 {
            s.create();
        } else {
            s.wire.request(3, 2, &[4, 5, 6]);
            s.pump();
        }
        assert_eq!(
            s.state.mappings,
            [if version == 1 {
                (None, None)
            } else {
                (Some(4), Some(5))
            }]
        );
        s.wire.request(6, 0, &[10, 384, (-512i32) as u32]);
        s.wire.request(6, 1, &[11, 20, 30, 100, 100]);
        s.wire.request(6, 2, &[12, 0x110, 1]);
        s.wire.request(6, 2, &[13, 0x110, 0]);
        s.wire.request(6, 4, &[]);
        s.pump();
        assert!(matches!(
            s.state.events.as_slice(),
            [
                Event::Motion {
                    time: 10,
                    dx: 1.5,
                    dy: -2.0
                },
                Event::Absolute {
                    time: 11,
                    x: 20,
                    y: 30,
                    x_extent: 100,
                    y_extent: 100
                },
                Event::Button {
                    time: 12,
                    button: 0x110,
                    pressed: true
                },
                Event::Button {
                    time: 13,
                    button: 0x110,
                    pressed: false
                },
                Event::Frame(None)
            ]
        ));
    }
}

#[test]
fn scroll_is_framed_preserves_axes_and_resets_source() {
    let mut s = Server::new(2);
    s.create();
    s.wire.request(6, 5, &[1]); // finger
    s.wire.request(6, 3, &[20, 0, 512]); // vertical 2.0
    s.wire.request(6, 7, &[20, 1, 256, (-2i32) as u32]); // horizontal, -2 detents
    s.wire.request(6, 6, &[21, 0]);
    s.pump();
    assert!(s.state.events.is_empty());
    s.wire.request(6, 4, &[]);
    s.pump();
    let Event::Frame(Some(frame)) = &s.state.events[0] else {
        panic!("missing scroll frame")
    };
    assert_eq!(frame.axis, (1.0, 2.0));
    assert_eq!(frame.v120, Some((-240, 0)));
    assert_eq!(frame.stop, (false, true));
    assert_eq!(frame.source, Some(AxisSource::Finger));
    assert_eq!(frame.time, 21);
    s.wire.request(6, 3, &[22, 0, 256]);
    s.wire.request(6, 4, &[]);
    s.pump();
    let Event::Frame(Some(frame)) = &s.state.events[1] else {
        panic!("missing second scroll frame")
    };
    assert_eq!(frame.source, None);
    assert_eq!(frame.v120, None);
    assert_eq!(frame.stop, (false, false));
}

#[test]
fn suspend_discards_input_and_pre_suspend_scroll() {
    let mut s = Server::new(2);
    s.create();
    s.wire.request(6, 3, &[10, 0, 256]);
    s.pump();
    s.state.active = false;
    s.state.epoch += 1;
    s.wire.request(6, 2, &[11, 0x110, 1]);
    s.wire.request(6, 0, &[11, 256, 0]);
    s.pump();
    assert!(s.state.events.is_empty());
    s.state.active = true;
    s.wire.request(6, 4, &[]);
    s.pump();
    assert!(matches!(s.state.events.as_slice(), [Event::Frame(None)]));
    s.wire.request(6, 1, &[12, 10, 10, 0, 100]);
    s.pump();
    assert_eq!(s.state.events.len(), 1);
}

#[test]
fn manager_destruction_keeps_pointers_alive_and_client_loss_cleans_them() {
    let mut s = Server::new(2);
    s.create();
    s.wire.request(3, 0, &[0, 7]);
    s.wire.request(3, 1, &[]);
    s.pump();
    assert!(s.state.destroyed.is_empty());
    s.wire.request(6, 2, &[1, 0x110, 1]);
    s.wire.request(6, 8, &[]);
    s.pump();
    assert_eq!(s.state.events.len(), 1);
    assert_eq!(s.state.destroyed.len(), 1);
    s.wire.socket.shutdown(std::net::Shutdown::Both).unwrap();
    s.pump();
    assert_eq!(s.state.destroyed.len(), 2);
}
