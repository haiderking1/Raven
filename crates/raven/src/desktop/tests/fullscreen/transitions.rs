use super::super::layers::assertions::focus;
use super::{assertions::*, fixture::*};

#[test]
fn mapped_enter_and_exit_require_committed_ack_not_just_request_or_ack() {
    let mut f = fixture();
    let buffer = f.buffer_sized(400, 600);
    let (_, first) = map(&mut f, buffer);
    let (top, second) = map(&mut f, buffer);
    let tiles = snapshot(&f, &[&first, &second]);
    let serial = configured(&request(&mut f, top, true), top, (800, 600), true);
    commit(&mut f, top);
    owner(&f, None);
    assert_eq!(snapshot(&f, &[&first, &second]), tiles);
    ack(&mut f, top, serial);
    owner(&f, None);
    assert!(f.state.window_is_visible(&first));
    commit(&mut f, top);
    owner(&f, Some(&second));
    layout(&f, &second, (0, 0), (800, 600));
    assert!(!f.state.window_is_visible(&first));

    let serial = configured(&request(&mut f, top, false), top, (400, 600), false);
    commit(&mut f, top);
    owner(&f, Some(&second));
    ack(&mut f, top, serial);
    owner(&f, Some(&second));
    assert!(!f.state.window_is_visible(&first));
    commit(&mut f, top);
    owner(&f, None);
    assert_eq!(snapshot(&f, &[&first, &second]), tiles);
    assert!(f.state.window_is_visible(&first));
}

#[test]
fn stale_committed_ack_cannot_apply_a_superseded_intent() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (top, window) = map(&mut f, buffer);
    let enter_serial = configured(&request(&mut f, top, true), top, (800, 600), true);
    ack(&mut f, top, enter_serial);
    let exit_serial = configured(&request(&mut f, top, false), top, (800, 600), false);
    commit(&mut f, top);
    owner(&f, None);
    let latest = configured(&request(&mut f, top, true), top, (800, 600), true);
    ack(&mut f, top, exit_serial);
    commit(&mut f, top);
    owner(&f, None);
    ack(&mut f, top, latest);
    owner(&f, None);
    commit(&mut f, top);
    owner(&f, Some(&window));
}

#[test]
fn later_activation_configure_ack_satisfies_pending_fullscreen_intent() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (top, window) = map(&mut f, buffer);
    let requested = configured(&request(&mut f, top, true), top, (800, 600), true);
    f.state.activate_window(None);
    f.dispatch();
    f.state.activate_window(Some(window.clone()));
    let newer = configured(&f.dispatch(), top, (800, 600), true);
    assert_ne!(requested, newer);
    ack(&mut f, top, newer);
    owner(&f, None);
    commit(&mut f, top);
    owner(&f, Some(&window));
}

#[test]
fn mapped_background_request_is_denied_without_focus_theft() {
    let mut f = fixture();
    let buffer = f.buffer_sized(400, 600);
    let (background, first) = map(&mut f, buffer);
    let (focused, second) = map(&mut f, buffer);
    let serial = configured(
        &request(&mut f, background, true),
        background,
        (400, 600),
        false,
    );
    ack(&mut f, background, serial);
    commit(&mut f, background);
    owner(&f, None);
    focus(&f, focused.surface);
    layout(&f, &first, (0, 0), (400, 600));
    layout(&f, &second, (400, 0), (400, 600));
    // Denial must not leave a latent intent that activates on a later focus change.
    f.state.activate_window(Some(first.clone()));
    f.dispatch();
    commit(&mut f, background);
    owner(&f, None);
}
