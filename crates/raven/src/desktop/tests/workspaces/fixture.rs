use super::super::{
    fixture::{Fixture, Toplevel},
    wire::Event,
};
use smithay::{
    desktop::Window,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_server::Resource,
};

pub(in crate::desktop::tests) fn fixture() -> Fixture {
    let mut f = Fixture::new();
    let output = Output::new(
        "workspace-test".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (800, 600).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    f.state.space_mut().map_output(&output, (0, 0));
    f.state.output = Some(output);
    f
}

pub(in crate::desktop::tests) fn map(f: &mut Fixture, buffer: u32) -> (Toplevel, Window) {
    let top = f.toplevel();
    f.configure(top);
    f.attach(top, buffer);
    (top, window(f, top))
}

pub(in crate::desktop::tests) fn window(f: &Fixture, top: Toplevel) -> Window {
    f.state
        .windows
        .iter()
        .find(|window| window.toplevel().unwrap().wl_surface().id().protocol_id() == top.surface)
        .expect("protocol toplevel has a compositor window")
        .clone()
}

pub(in crate::desktop::tests) fn attach(f: &mut Fixture, top: Toplevel, buffer: u32) -> Vec<Event> {
    f.wire.request(top.surface, 1, &[buffer, 0, 0]);
    f.wire.request(top.surface, 6, &[]);
    f.dispatch()
}

pub(in crate::desktop::tests) fn frame(f: &mut Fixture, top: Toplevel) -> u32 {
    let callback = f.id();
    f.wire.request(top.surface, 3, &[callback]);
    f.wire.request(top.surface, 6, &[]);
    f.dispatch();
    callback
}
