use super::{assertions::tiled, fixture::fixture};

#[test]
fn mapped_maximize_requests_answer_with_the_existing_tile_even_when_refused() {
    let mut f = fixture(true);
    let top = f.toplevel();
    f.configure(top);
    let buffer = f.buffer_sized(501, 601);
    f.attach(top, buffer);
    let stacking: Vec<_> = f.state.space().elements().cloned().collect();

    for (opcode, args) in [(9, vec![]), (10, vec![])] {
        f.wire.request(top.role, opcode, &args);
        let events = f.dispatch();
        let serial = tiled(&events, top, (501, 601));
        f.wire.request(top.xdg, 4, &[serial]);
        f.wire.request(top.surface, 6, &[]);
        let events = f.dispatch();
        assert!(!events.iter().any(|e| e.object == top.role && e.opcode == 0));
        assert_eq!(
            f.state.space().elements().cloned().collect::<Vec<_>>(),
            stacking
        );
        assert_eq!(
            f.state.space().element_location(stacking.last().unwrap()),
            Some((500, 0).into())
        );
    }
}
