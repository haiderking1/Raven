use super::{assertions::*, fixture::*};

#[test]
fn shortcut_action_toggles_requested_intent_before_client_commits() {
    let mut f = fixture();
    let buffer = f.buffer();
    let (top, window) = map(&mut f, buffer);
    // Exercise the compositor action with the real xdg configure/ACK path.
    f.state.toggle_fullscreen();
    let first = configured(&f.dispatch(), top, (800, 600), true);
    f.state.toggle_fullscreen();
    let cancelled = configured(&f.dispatch(), top, (800, 600), false);
    ack(&mut f, top, first);
    commit(&mut f, top);
    owner(&f, None);
    ack(&mut f, top, cancelled);
    commit(&mut f, top);
    owner(&f, None);
    f.state.toggle_fullscreen();
    let serial = configured(&f.dispatch(), top, (800, 600), true);
    ack(&mut f, top, serial);
    owner(&f, None);
    commit(&mut f, top);
    owner(&f, Some(&window));
    f.state.toggle_fullscreen();
    let serial = configured(&f.dispatch(), top, (800, 600), false);
    ack(&mut f, top, serial);
    owner(&f, Some(&window));
    commit(&mut f, top);
    owner(&f, None);
}
