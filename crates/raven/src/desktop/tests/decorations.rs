use super::{
    fixture::{Fixture, Toplevel},
    wire::{Event, string, word},
};

fn bind_manager(f: &mut Fixture) -> u32 {
    let registry = f.id();
    let manager = f.id();
    f.wire.request(1, 1, &[registry]);
    let events = f.dispatch();
    let interface = "zxdg_decoration_manager_v1";
    let global = events
        .iter()
        .find(|event| {
            event.object == registry && event.opcode == 0 && {
                let len = word(&event.args[4..]) as usize;
                &event.args[8..8 + len - 1] == interface.as_bytes()
            }
        })
        .expect("Raven advertises XDG decoration negotiation");
    let mut args = word(&global.args).to_ne_bytes().to_vec();
    args.extend(string(interface));
    args.extend(1_u32.to_ne_bytes());
    args.extend(manager.to_ne_bytes());
    f.wire.bytes(registry, 0, &args, None);
    f.dispatch();
    manager
}

fn acknowledge(f: &mut Fixture, top: Toplevel, events: &[Event]) {
    let event = events
        .iter()
        .rev()
        .find(|event| event.object == top.xdg && event.opcode == 0)
        .expect("decoration request receives an XDG configure");
    f.wire.request(top.xdg, 4, &[word(&event.args)]);
}

#[test]
fn server_side_policy_survives_client_preferences_and_decoration_recreation() {
    let mut f = Fixture::new();
    let manager = bind_manager(&mut f);
    let top = f.toplevel();
    let decoration = f.id();
    f.wire.request(manager, 1, &[decoration, top.role]);
    f.wire.request(decoration, 1, &[1]); // ClientSide is a preference, not a command.
    let events = f.dispatch();
    assert!(
        !events
            .iter()
            .any(|event| [decoration, top.xdg].contains(&event.object) && event.opcode == 0)
    );

    f.wire.request(top.surface, 6, &[]);
    let events = f.dispatch();
    let mode_index = events
        .iter()
        .position(|event| event.object == decoration && event.opcode == 0)
        .unwrap();
    let configure_index = events
        .iter()
        .position(|event| event.object == top.xdg && event.opcode == 0)
        .unwrap();
    assert_eq!(word(&events[mode_index].args), 2); // ServerSide
    assert!(mode_index < configure_index);
    acknowledge(&mut f, top, &events);

    for (opcode, args) in [(1, vec![1]), (2, vec![])] {
        f.wire.request(decoration, opcode, &args);
        let events = f.dispatch();
        assert!(
            events
                .iter()
                .filter(|event| event.object == decoration && event.opcode == 0)
                .all(|event| word(&event.args) == 2)
        );
        acknowledge(&mut f, top, &events);
    }

    // A new decoration on an already-configured toplevel needs its own mode
    // event even though the server's chosen mode hasn't changed.
    f.wire.request(decoration, 0, &[]);
    let replacement = f.id();
    f.wire.request(manager, 1, &[replacement, top.role]);
    let events = f.dispatch();
    let mode = events
        .iter()
        .find(|event| event.object == replacement && event.opcode == 0)
        .unwrap();
    assert_eq!(word(&mode.args), 2);
    acknowledge(&mut f, top, &events);
}
