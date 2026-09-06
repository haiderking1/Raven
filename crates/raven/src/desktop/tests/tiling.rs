use super::{
    fixture::{Fixture, Toplevel},
    wire::{Event, word},
};
use smithay::{
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{wayland_protocols::xdg::shell::server::xdg_toplevel, wayland_server::Resource},
};

fn configure_size(events: &[Event], top: Toplevel) -> (u32, u32) {
    let event = events
        .iter()
        .rev()
        .find(|event| event.object == top.role && event.opcode == 0)
        .expect("toplevel received a size configure");
    let states: Vec<_> = event.args[12..].chunks_exact(4).map(word).collect();
    for edge in [
        xdg_toplevel::State::TiledLeft,
        xdg_toplevel::State::TiledRight,
        xdg_toplevel::State::TiledTop,
        xdg_toplevel::State::TiledBottom,
    ] {
        assert!(states.contains(&(edge as u32)));
    }
    (word(&event.args), word(&event.args[4..]))
}

fn attach(f: &mut Fixture, top: Toplevel, buffer: u32) -> Vec<Event> {
    f.wire.request(top.surface, 1, &[buffer, 0, 0]);
    f.wire.request(top.surface, 6, &[]);
    f.dispatch()
}

#[test]
fn tiled_configures_reflow_on_unmap_without_following_focus_order() {
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
            size: (1001, 601).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    f.state.space_mut().map_output(&output, (0, 0));
    f.state.output = Some(output);

    let first = f.toplevel();
    f.wire.request(first.surface, 6, &[]);
    let events = f.dispatch();
    assert_eq!(configure_size(&events, first), (1001, 601));
    let serial = events
        .iter()
        .find(|event| event.object == first.xdg && event.opcode == 0)
        .unwrap();
    f.wire.request(first.xdg, 4, &[word(&serial.args)]);
    let buffer = f.buffer();
    f.attach(first, buffer);
    let first_window = f.state.space().elements().next().unwrap().clone();

    let second = f.toplevel();
    f.configure(second);
    let events = attach(&mut f, second, buffer);
    assert_eq!(configure_size(&events, first), (500, 601));
    assert_eq!(configure_size(&events, second), (501, 601));
    let second_window = f
        .state
        .space()
        .elements()
        .find(|window| window.toplevel().unwrap().wl_surface().id().protocol_id() == second.surface)
        .unwrap()
        .clone();
    assert_eq!(
        f.state.space().element_location(&first_window),
        Some((0, 0).into())
    );
    assert_eq!(
        f.state.space().element_location(&second_window),
        Some((500, 0).into())
    );

    // Raising the master changes z-order, not its tile. An unchanged layout
    // must not repeatedly configure clients and provoke redraw loops.
    f.state.activate_window(Some(first_window.clone()));
    f.dispatch();
    f.state.retile_workspace(f.state.workspaces.active);
    let events = f.dispatch();
    assert!(
        !events
            .iter()
            .any(|event| [first.role, second.role].contains(&event.object) && event.opcode == 0)
    );
    assert_eq!(f.state.space().elements().next_back(), Some(&first_window));
    assert_eq!(
        f.state.space().element_location(&first_window),
        Some((0, 0).into())
    );
    assert_eq!(
        f.state.space().element_location(&second_window),
        Some((500, 0).into())
    );

    let events = attach(&mut f, second, 0);
    assert_eq!(configure_size(&events, first), (1001, 601));
    assert_eq!(f.state.space().elements().count(), 1);
    f.configure(second);
    let events = attach(&mut f, second, buffer);
    assert_eq!(configure_size(&events, first), (500, 601));
    assert_eq!(
        f.state.space().element_location(&second_window),
        Some((500, 0).into())
    );

    f.wire.request(second.role, 0, &[]);
    f.wire.request(second.xdg, 0, &[]);
    f.wire.request(second.surface, 0, &[]);
    let events = f.dispatch();
    assert_eq!(configure_size(&events, first), (1001, 601));
    assert_eq!(f.state.space().elements().count(), 1);
}
