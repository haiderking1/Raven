use crate::desktop::tests::{fixture::Fixture, wire::string};
use smithay::{
    desktop::Window,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_server::Resource,
};
pub(super) fn fixture() -> Fixture {
    let mut f = Fixture::new();
    let output = Output::new(
        "screenshot-test".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (1280, 800).into(),
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
pub(super) fn app(f: &mut Fixture, id: &str) -> Window {
    let top = f.toplevel();
    f.wire.bytes(top.role, 3, &string(id), None);
    f.configure(top);
    let buffer = f.buffer();
    f.attach(top, buffer);
    let window = f
        .state
        .windows
        .iter()
        .find(|w| {
            w.toplevel()
                .is_some_and(|t| t.wl_surface().id().protocol_id() == top.surface)
        })
        .unwrap()
        .clone();
    f.state.activate_window(Some(window.clone()));
    window
}
