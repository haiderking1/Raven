use super::super::{assertions::*, fixture::*};
use crate::desktop::tests::{
    layers::assertions::{focus, hit},
    popups::fixture::map_popup,
};
use smithay::reexports::wayland_server::Resource;

#[test]
fn fullscreen_popup_is_hittable_but_hidden_parent_popup_cannot_receive_focus() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (background, hidden) = map(&mut f, buffer);
    let hidden_popup = map_popup(&mut f, background).surface;
    hit(&f, (30.0, 30.0), hidden_popup);
    let (top, window) = map(&mut f, buffer);
    enter(&mut f, top);
    let visible_popup = map_popup(&mut f, top).surface;
    // Popup geometry is relative to the centered parent's window geometry.
    hit(&f, (380.0, 280.0), visible_popup);
    assert_eq!(
        f.state.surface_under((380.0, 280.0).into()).unwrap().1,
        (375.0, 275.0).into()
    );
    assert!(!f.state.window_is_visible(&hidden));
    assert!(f.state.surface_under((30.0, 30.0).into()).is_none());
    f.state.focus_window_at((30.0, 30.0).into());
    focus(&f, top.surface);
    f.state.focus_window_at((380.0, 280.0).into());
    focus(&f, top.surface);
    owner(&f, Some(&window));
    f.state.switch_workspace(1);
    assert!(f.state.surface_under((380.0, 280.0).into()).is_none());
    assert!(!f.state.window_is_visible(&window));
    assert_ne!(
        f.state
            .seat
            .get_keyboard()
            .unwrap()
            .current_focus()
            .map(|surface| surface.id().protocol_id()),
        Some(visible_popup)
    );
}

#[test]
fn popup_keyboard_focus_keeps_its_parent_eligible_for_fullscreen() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (top, window) = map(&mut f, buffer);
    let popup_id = map_popup(&mut f, top).surface;
    let (popup_surface, _) = f.state.surface_under((30.0, 30.0).into()).unwrap();
    assert_eq!(popup_surface.id().protocol_id(), popup_id);
    let keyboard = f.state.seat.get_keyboard().unwrap();
    keyboard.set_focus(
        &mut f.state,
        Some(popup_surface),
        smithay::utils::SERIAL_COUNTER.next_serial(),
    );
    f.dispatch();
    focus(&f, popup_id);
    assert_eq!(f.state.focused_window(), Some(window.clone()));
    enter(&mut f, top);
    owner(&f, Some(&window));
    assert_eq!(f.state.focused_window(), Some(window));
    hit(&f, (380.0, 280.0), popup_id);
}

#[test]
fn reactive_popup_is_reconstrained_after_fullscreen_parent_placement_changes() {
    use crate::desktop::tests::{popups::fixture::map_reactive_popup, wire::word};
    let mut f = fixture();
    let buffer = f.buffer_sized(400, 600);
    map(&mut f, buffer);
    map(&mut f, buffer);
    let (top, window) = map(&mut f, buffer);
    let popup = map_reactive_popup(&mut f, top, [360, 560, 40, 40]);
    hit(&f, (770.0, 585.0), popup.surface);
    let serial = configured(&request(&mut f, top, true), top, (800, 600), true);
    ack(&mut f, top, serial);
    let events = commit(&mut f, top);
    owner(&f, Some(&window));
    let configure = events
        .iter()
        .find(|e| e.object == popup.role && e.opcode == 0)
        .expect("reactive popup configure");
    assert_eq!(
        configure.args.chunks_exact(4).map(word).collect::<Vec<_>>(),
        vec![365, 570, 30, 20]
    );
    let serial = events
        .iter()
        .find(|e| e.object == popup.xdg && e.opcode == 0)
        .unwrap();
    f.wire.request(popup.xdg, 4, &[word(&serial.args)]);
    f.wire.request(popup.surface, 6, &[]);
    f.dispatch();
    hit(&f, (570.0, 575.0), popup.surface);
    f.state.refresh();
    assert!(
        !f.dispatch()
            .iter()
            .any(|e| e.object == popup.role && e.opcode == 0),
        "unchanged placement must not repeat configures"
    );
}
