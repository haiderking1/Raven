use super::super::{
    fixture::{Fixture, Toplevel},
    wire::{Event, word},
};
use smithay::{desktop::Window, reexports::wayland_server::Resource};

pub(super) fn layout(f: &Fixture, index: usize, tiles: &[(&Window, (i32, i32))]) {
    let space = &f.state.workspaces.entries[index].space;
    assert_eq!(space.elements().count(), tiles.len(), "workspace {index}");
    for &(window, location) in tiles {
        assert_eq!(f.state.workspaces.index_of(window), Some(index));
        assert_eq!(space.element_location(window), Some(location.into()));
    }
}

pub(super) fn focus(f: &Fixture, index: usize, expected: Option<&Window>) {
    assert_eq!(f.state.workspaces.active, index);
    assert_eq!(f.state.focused_window().as_ref(), expected);
    assert_eq!(f.state.workspaces.entries[index].focused.as_ref(), expected);
    assert_eq!(
        f.state.seat.get_keyboard().unwrap().current_focus(),
        expected.map(|window| window.toplevel().unwrap().wl_surface().clone())
    );
}

pub(super) fn pointer(f: &Fixture, expected: Option<Toplevel>) {
    assert_eq!(
        f.state
            .seat
            .get_pointer()
            .unwrap()
            .current_focus()
            .map(|surface| surface.id().protocol_id()),
        expected.map(|top| top.surface)
    );
    assert_eq!(
        f.state
            .surface_under(f.state.pointer_location)
            .map(|(surface, _)| surface.id().protocol_id()),
        expected.map(|top| top.surface)
    );
}

pub(super) fn output_membership(f: &mut Fixture) {
    f.state.refresh();
    let output = f.state.output.as_ref().unwrap();
    for (index, workspace) in f.state.workspaces.entries.iter().enumerate() {
        let expected = if index == f.state.workspaces.active {
            vec![output.clone()]
        } else {
            vec![]
        };
        assert_eq!(
            workspace.space.outputs().cloned().collect::<Vec<_>>(),
            expected
        );
        for window in workspace.space.elements() {
            assert_eq!(workspace.space.outputs_for_element(window), expected);
        }
    }
}

pub(super) fn size(events: &[Event], top: Toplevel, expected: (u32, u32)) {
    let configure = events
        .iter()
        .rev()
        .find(|event| event.object == top.role && event.opcode == 0)
        .expect("reflow sends a toplevel configure");
    assert_eq!(
        (word(&configure.args), word(&configure.args[4..])),
        expected
    );
}

pub(super) fn frame_done(events: &[Event], callback: u32) -> bool {
    events
        .iter()
        .any(|event| event.object == callback && event.opcode == 0)
}
