use super::fixture::Fixture;
use smithay::{
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_server::Resource,
};
use std::time::Duration;

#[test]
fn configure_map_unmap_remap_restores_focus() {
    let mut f = Fixture::new();
    let first = f.toplevel();
    assert!(
        !f.dispatch()
            .iter()
            .any(|e| e.object == first.xdg && e.opcode == 0)
    );
    f.configure(first);
    assert_eq!(f.state.space().elements().count(), 0);
    let buffer = f.buffer();
    f.attach(first, buffer);
    assert_eq!(f.state.space().elements().count(), 1);
    let second = f.toplevel();
    f.configure(second);
    f.attach(second, buffer);
    let keyboard = f.state.seat.get_keyboard().unwrap();
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        second.surface
    );
    f.attach(second, 0);
    assert_eq!(f.state.space().elements().count(), 1);
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        first.surface
    );
    f.configure(second);
    f.attach(second, buffer);
    assert_eq!(f.state.space().elements().count(), 2);
    assert_eq!(
        keyboard.current_focus().unwrap().id().protocol_id(),
        second.surface
    );
}

#[test]
fn hit_test_returns_buffer_origin_not_window_geometry_origin() {
    let mut f = Fixture::new();
    let top = f.toplevel();
    f.configure(top);
    f.wire.request(top.xdg, 3, &[10, 12, 80, 70]);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let window = f.state.space().elements().next().unwrap().clone();
    f.state.space_mut().map_element(window, (100, 100), false);
    let (surface, origin) = f.state.surface_under((110.0, 110.0).into()).unwrap();
    assert_eq!(surface.id().protocol_id(), top.surface);
    assert_eq!(origin, (90.0, 88.0).into());
    assert!(f.state.surface_under((250.0, 250.0).into()).is_none());
}

#[test]
fn frame_callbacks_require_window_overlap_with_output() {
    let mut f = Fixture::new();
    let output = Output::new(
        "test".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (800, 600).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    f.state.space_mut().map_output(&output, (0, 0));
    f.state.output = Some(output);
    let top = f.toplevel();
    f.configure(top);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let window = f.state.space().elements().next().unwrap().clone();
    let callback = f.id();
    f.wire.request(top.surface, 3, &[callback]);
    f.wire.request(top.surface, 6, &[]);
    f.dispatch();
    f.state
        .space_mut()
        .map_element(window.clone(), (900, 0), false);
    f.state.refresh();
    f.state.send_frames(Duration::from_millis(10));
    assert!(
        !f.dispatch()
            .iter()
            .any(|e| e.object == callback && e.opcode == 0)
    );
    f.state.space_mut().map_element(window, (0, 0), false);
    f.state.refresh();
    f.state.send_frames(Duration::from_millis(20));
    assert!(
        f.dispatch()
            .iter()
            .any(|e| e.object == callback && e.opcode == 0)
    );
}
