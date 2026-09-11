use super::super::{
    fixture::{Fixture, Toplevel},
    wire::{Event, word},
};
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

pub(super) fn configured(events: &[Event], top: Toplevel, size: (u32, u32)) -> u32 {
    let config = events
        .iter()
        .rev()
        .find(|e| e.object == top.role && e.opcode == 0)
        .expect("real xdg_toplevel size configure");
    assert_eq!((word(&config.args), word(&config.args[4..])), size);
    word(
        &events
            .iter()
            .rev()
            .find(|e| e.object == top.xdg && e.opcode == 0)
            .expect("configure serial")
            .args,
    )
}

pub(super) fn ack(f: &mut Fixture, top: Toplevel, serial: u32) {
    f.wire.request(top.xdg, 4, &[serial]);
}

pub(super) fn allocation(f: &Fixture, window: &Window, frame: (i32, i32, i32, i32), border: i32) {
    let (x, y, w, h) = frame;
    assert_eq!(
        f.state.window_frame_geometry(window),
        Some(Rectangle::<i32, Logical>::new((x, y).into(), (w, h).into()))
    );
    let client = Rectangle::new(
        (x + border, y + border).into(),
        (w - 2 * border, h - 2 * border).into(),
    );
    assert_eq!(f.state.window_client_geometry(window), Some(client));
    assert!(client.size.w > 0 && client.size.h > 0);
}
