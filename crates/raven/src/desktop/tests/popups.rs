use super::{fixture::Fixture, wire::word};

#[test]
fn popup_gets_initial_configure_and_reposition_acknowledgment() {
    let mut f = Fixture::new();
    let top = f.toplevel();
    f.configure(top);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let positioner = f.id();
    f.wire.request(5, 1, &[positioner]);
    f.wire.request(positioner, 1, &[30, 20]);
    f.wire.request(positioner, 2, &[0, 0, 80, 70]);
    let surface = f.id();
    let xdg = f.id();
    let popup = f.id();
    f.wire.request(3, 0, &[surface]);
    f.wire.request(5, 2, &[xdg, surface]);
    f.wire.request(xdg, 2, &[popup, top.xdg, positioner]);
    assert!(
        !f.dispatch()
            .iter()
            .any(|e| e.object == xdg && e.opcode == 0)
    );
    f.wire.request(surface, 6, &[]);
    let events = f.dispatch();
    assert!(events.iter().any(|e| e.object == popup && e.opcode == 0));
    let configure = events
        .iter()
        .find(|e| e.object == xdg && e.opcode == 0)
        .unwrap();
    f.wire.request(xdg, 4, &[word(&configure.args)]);
    f.wire.request(surface, 1, &[buffer, 0, 0]);
    f.wire.request(surface, 6, &[]);
    f.dispatch();
    f.wire.request(positioner, 7, &[15, 10]);
    f.wire.request(popup, 2, &[positioner, 42]);
    let events = f.dispatch();
    assert!(
        events
            .iter()
            .any(|e| e.object == popup && e.opcode == 2 && word(&e.args) == 42)
    );
    assert!(events.iter().any(|e| e.object == xdg && e.opcode == 0));
}
