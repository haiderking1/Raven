use super::super::wire::Event;
pub(super) use super::super::{
    fixture::{Fixture, Toplevel},
    workspaces::fixture::{fixture, frame, map, window},
};
use super::assertions::configured;

pub(super) fn request(f: &mut Fixture, top: Toplevel, fullscreen: bool) -> Vec<Event> {
    if fullscreen {
        f.wire.request(top.role, 11, &[0]);
    } else {
        f.wire.request(top.role, 12, &[]);
    }
    f.dispatch()
}

pub(super) fn ack(f: &mut Fixture, top: Toplevel, serial: u32) {
    f.wire.request(top.xdg, 4, &[serial]);
    f.dispatch();
}

pub(super) fn commit(f: &mut Fixture, top: Toplevel) -> Vec<Event> {
    f.wire.request(top.surface, 6, &[]);
    f.dispatch()
}

pub(super) fn enter(f: &mut Fixture, top: Toplevel) {
    let events = request(f, top, true);
    let serial = configured(&events, top, (800, 600), true);
    ack(f, top, serial);
    commit(f, top);
}

pub(super) fn startup(f: &mut Fixture, buffer: u32) -> (Toplevel, smithay::desktop::Window) {
    let (top, window, _) = startup_with_events(f, buffer);
    (top, window)
}

pub(super) fn startup_with_events(
    f: &mut Fixture,
    buffer: u32,
) -> (Toplevel, smithay::desktop::Window, Vec<Event>) {
    let top = f.toplevel();
    let events = request(f, top, true);
    super::assertions::no_configure(&events, top);
    let events = commit(f, top);
    let serial = configured(&events, top, (800, 600), true);
    ack(f, top, serial);
    let events = super::super::workspaces::fixture::attach(f, top, buffer);
    (top, window(f, top), events)
}

pub(super) fn destroy(f: &mut Fixture, top: Toplevel) {
    f.wire.request(top.role, 0, &[]);
    f.wire.request(top.xdg, 0, &[]);
    f.wire.request(top.surface, 0, &[]);
    f.dispatch();
}
