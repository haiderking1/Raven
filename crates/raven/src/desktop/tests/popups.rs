pub(super) mod fixture;

use super::{fixture::Fixture, wire::word};
use fixture::map_popup;

#[test]
fn popup_gets_initial_configure_and_reposition_acknowledgment() {
    let mut f = Fixture::new();
    let top = f.toplevel();
    f.configure(top);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let popup = map_popup(&mut f, top);
    f.wire.request(popup.positioner, 7, &[15, 10]);
    f.wire.request(popup.role, 2, &[popup.positioner, 42]);
    let events = f.dispatch();
    assert!(
        events
            .iter()
            .any(|e| e.object == popup.role && e.opcode == 2 && word(&e.args) == 42)
    );
    assert!(
        events
            .iter()
            .any(|e| e.object == popup.xdg && e.opcode == 0)
    );
}
