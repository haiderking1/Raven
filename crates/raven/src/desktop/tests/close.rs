use super::fixture::Fixture;

#[test]
fn close_requests_target_keyboard_focus_and_wait_for_the_client() {
    let mut f = Fixture::new();
    let first = f.toplevel();
    f.configure(first);
    let buffer = f.buffer();
    f.attach(first, buffer);
    let first_window = f.state.space().elements().next().unwrap().clone();
    let second = f.toplevel();
    f.configure(second);
    f.attach(second, buffer);

    f.state.close_focused_window();
    let events = f.dispatch();
    let closed: Vec<_> = events
        .iter()
        .filter(|event| [first.role, second.role].contains(&event.object) && event.opcode == 1)
        .map(|event| event.object)
        .collect();
    assert_eq!(closed, vec![second.role]);
    assert_eq!(
        f.state.space().elements().count(),
        2,
        "requesting close must not forcibly unmap a client"
    );

    f.state.activate_window(Some(first_window));
    f.state.close_focused_window();
    let events = f.dispatch();
    let closed: Vec<_> = events
        .iter()
        .filter(|event| [first.role, second.role].contains(&event.object) && event.opcode == 1)
        .map(|event| event.object)
        .collect();
    assert_eq!(closed, vec![first.role]);

    f.state.activate_window(None);
    f.state.close_focused_window();
    let events = f.dispatch();
    assert!(
        !events
            .iter()
            .any(|event| [first.role, second.role].contains(&event.object) && event.opcode == 1)
    );
}
