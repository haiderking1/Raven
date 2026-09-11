mod support;

use crate::desktop::tests::fixture::Fixture;
use smithay::backend::input::ButtonState;
use support::*;

#[test]
fn workspace_wire_transactions_and_lifecycle() {
    let mut f = Fixture::new();
    let output = output(&mut f);
    let global = global(&mut f, "ext_workspace_manager_v1");
    let (first, initial) = Manager::bind(&mut f, global);
    assert_eq!(first.workspaces.len(), 10);
    assert_eq!(
        first.states(&initial),
        (0..10).map(|i| (i, u32::from(i == 0))).collect::<Vec<_>>()
    );
    assert!(
        initial
            .iter()
            .any(|e| e.object == first.group && e.opcode == 0 && word(&e.args) == 0)
    );
    for (index, id) in first.workspaces.iter().enumerate() {
        let properties: Vec<_> = initial.iter().filter(|e| e.object == *id).collect();
        assert_eq!(
            properties.iter().map(|e| e.opcode).collect::<Vec<_>>(),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            text(&properties[0].args),
            format!("raven-workspace-{}", index + 1)
        );
        assert_eq!(text(&properties[1].args), (index + 1).to_string());
        assert_eq!(
            properties[2].args,
            [4u32.to_ne_bytes(), (index as u32).to_ne_bytes()].concat()
        );
        assert_eq!(word(&properties[4].args), 1, "only Activate is supported");
        assert!(
            initial
                .iter()
                .any(|e| e.object == first.group && e.opcode == 3 && word(&e.args) == *id)
        );
    }
    first.done(&initial);

    // Manager-before-output ordering is how some panels discover globals.
    let output_global = support::global(&mut f, "wl_output");
    let (output_one, _) = bind(&mut f, output_global, "wl_output", 4);
    f.state.refresh_workspace_protocol();
    let events = dispatch(&mut f);
    assert!(
        events
            .iter()
            .any(|e| e.object == first.group && e.opcode == 1 && word(&e.args) == output_one)
    );
    first.done(&events);
    let (output_two, _) = bind(&mut f, output_global, "wl_output", 4);
    f.state.refresh_workspace_protocol();
    let events = dispatch(&mut f);
    assert_eq!(
        events
            .iter()
            .filter(|e| e.object == first.group && e.opcode == 1)
            .map(|e| word(&e.args))
            .collect::<Vec<_>>(),
        [output_two]
    );
    first.done(&events);
    let (second, events) = Manager::bind(&mut f, global);
    assert_eq!(
        events
            .iter()
            .filter(|e| e.object == second.group && e.opcode == 1)
            .count(),
        2
    );
    for (index, id) in second.workspaces.iter().enumerate() {
        assert!(events.iter().any(|e| e.object == *id
            && e.opcode == 0
            && text(&e.args) == format!("raven-workspace-{}", index + 1)));
    }
    second.done(&events);
    f.state.refresh_workspace_protocol();
    assert!(
        dispatch(&mut f).is_empty(),
        "unchanged snapshots emit no events"
    );

    // Pending requests belong to a manager, not to the client connection.
    f.wire.request(first.workspaces[1], 1, &[]);
    f.wire.request(first.workspaces[2], 1, &[]);
    assert!(dispatch(&mut f).is_empty());
    assert_eq!(f.state.workspaces.active, 0);
    // Unadvertised operations must not mutate the fixed workspace set.
    f.wire.request(second.workspaces[0], 2, &[]); // deactivate
    f.wire.request(second.workspaces[0], 4, &[]); // remove
    f.wire.request(second.workspaces[0], 3, &[second.group]); // assign
    f.wire.bytes(second.group, 0, &string("extra"), None); // create
    f.wire.request(second.id, 0, &[]);
    let events = dispatch(&mut f);
    assert_eq!(f.state.workspaces.active, 0);
    assert!(!events.iter().any(|e| first.owns(e)));
    assert_eq!(events.iter().filter(|e| second.owns(e)).count(), 1);
    second.done(&events);
    f.wire.request(first.id, 0, &[]);
    let events = dispatch(&mut f);
    assert_eq!(f.state.workspaces.active, 2);
    assert_eq!(first.states(&events), [(0, 0), (2, 1)]);
    assert_eq!(second.states(&events), [(0, 0), (2, 1)]);
    first.done(&events);
    second.done(&events);

    // A real Smithay implicit pointer grab denies the switch, and commit reports
    // the actual inactive state rather than promising a later activation.
    button(&mut f, ButtonState::Pressed);
    assert!(f.state.seat.get_pointer().unwrap().is_grabbed());
    f.wire.request(first.workspaces[3], 1, &[]);
    f.wire.request(first.id, 0, &[]);
    let events = dispatch(&mut f);
    assert_eq!(f.state.workspaces.active, 2);
    assert_eq!(first.states(&events), [(3, 0)]);
    assert!(!events.iter().any(|e| second.owns(e)));
    first.done(&events);
    button(&mut f, ButtonState::Released);
    assert!(!f.state.seat.get_pointer().unwrap().is_grabbed());
    f.wire.request(first.id, 0, &[]);
    let events = dispatch(&mut f);
    assert_eq!(f.state.workspaces.active, 2, "denied requests are consumed");
    assert!(first.states(&events).is_empty());
    first.done(&events);

    f.state.switch_workspace(9);
    f.state.refresh_workspace_protocol();
    let events = dispatch(&mut f);
    assert_eq!(first.states(&events), [(2, 0), (9, 1)]);
    assert_eq!(second.states(&events), [(2, 0), (9, 1)]);
    first.done(&events);
    second.done(&events);
    f.state.refresh_workspace_protocol();
    assert!(dispatch(&mut f).is_empty());

    // Output removal must reference every still-live wl_output instance.
    f.state.output = None;
    f.state.refresh_workspace_protocol();
    let events = dispatch(&mut f);
    for manager in [&first, &second] {
        let mut left: Vec<_> = events
            .iter()
            .filter(|e| e.object == manager.group && e.opcode == 2)
            .map(|e| word(&e.args))
            .collect();
        left.sort_unstable();
        assert_eq!(left, [output_one, output_two]);
        manager.done(&events);
    }
    f.wire.request(output_one, 0, &[]); // wl_output.release
    dispatch(&mut f);
    f.state.output = Some(output);
    f.state.refresh_workspace_protocol();
    let events = dispatch(&mut f);
    for manager in [&first, &second] {
        assert_eq!(
            events
                .iter()
                .filter(|e| e.object == manager.group && e.opcode == 1)
                .map(|e| word(&e.args))
                .collect::<Vec<_>>(),
            [output_two]
        );
        manager.done(&events);
    }

    f.wire.request(first.workspaces[4], 1, &[]);
    f.wire.request(first.id, 1, &[]); // stop discards its pending transaction
    let events = dispatch(&mut f);
    assert_eq!(
        events
            .iter()
            .filter(|e| first.owns(e))
            .map(|e| (e.object, e.opcode))
            .collect::<Vec<_>>(),
        [(first.id, 3)]
    );
    assert_eq!(f.state.workspace_protocol.subscriptions.len(), 1);
    f.wire.request(first.workspaces[5], 1, &[]); // stopped children are inert
    for id in &first.workspaces {
        f.wire.request(*id, 0, &[]);
    }
    f.wire.request(first.group, 1, &[]);
    dispatch(&mut f);
    f.state.switch_workspace(0);
    f.state.refresh_workspace_protocol();
    let events = dispatch(&mut f);
    assert!(!events.iter().any(|e| first.owns(e)));
    assert_eq!(second.states(&events), [(0, 1), (9, 0)]);
    second.done(&events);

    f.wire.request(second.workspaces[6], 1, &[]);
    f.wire.request(second.workspaces[6], 0, &[]);
    f.wire.request(second.group, 1, &[]);
    dispatch(&mut f);
    let subscription = &f.state.workspace_protocol.subscriptions[0];
    assert!(subscription.group.is_none());
    assert!(subscription.outputs.is_empty());
    assert!(subscription.workspaces[6].is_none());
    assert!(subscription.pending.is_none());
    f.wire.request(second.workspaces[7], 1, &[]);
    dispatch(&mut f);
    f.disconnect();
    assert!(f.state.workspace_protocol.subscriptions.is_empty());
}
