use super::super::{assertions::configured, fixture::*};
use crate::desktop::tests::wire::{Event, word};
use smithay::{
    desktop::Window, reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State,
};

pub(super) fn parent(f: &mut Fixture, child: Toplevel, parent: Option<Toplevel>) -> Vec<Event> {
    f.wire
        .request(child.role, 1, &[parent.map_or(0, |top| top.role)]);
    f.dispatch()
}

pub(super) fn dialog(f: &mut Fixture, owner: Toplevel, buffer: u32) -> (Toplevel, Window) {
    let top = f.toplevel();
    super::super::assertions::no_configure(&parent(f, top, Some(owner)), top);
    let events = commit(f, top);
    let serial = configured(&events, top, (0, 0), false);
    not_tiled(&events, top);
    ack(f, top, serial);
    f.attach(top, buffer);
    (top, window(f, top))
}

pub(super) fn not_tiled(events: &[Event], top: Toplevel) {
    for config in events
        .iter()
        .filter(|e| e.object == top.role && e.opcode == 0)
    {
        let (states, remainder) = config.args[12..].as_chunks::<4>();
        assert!(remainder.is_empty());
        let states: Vec<_> = states.iter().map(|state| word(state)).collect();
        for flag in [
            State::Fullscreen,
            State::Maximized,
            State::TiledLeft,
            State::TiledRight,
            State::TiledTop,
            State::TiledBottom,
        ] {
            assert!(
                !states.contains(&(flag as u32)),
                "dialog must not advertise {flag:?}"
            );
        }
    }
}
