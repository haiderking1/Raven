use super::super::{
    fixture::{Fixture, Toplevel},
    wire::{string, word},
};
use smithay::{
    desktop::Window,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_server::Resource,
};

pub(in crate::desktop::tests) fn fixture() -> (Fixture, u32) {
    let mut f = Fixture::new();
    let output = Output::new(
        "layer-test".into(),
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

    let registry = f.id();
    f.wire.request(1, 1, &[registry]);
    let events = f.dispatch();
    let interface = "zwlr_layer_shell_v1";
    let global = events
        .iter()
        .find(|event| {
            event.object == registry && event.opcode == 0 && {
                let len = word(&event.args[4..]) as usize;
                &event.args[8..8 + len - 1] == interface.as_bytes()
            }
        })
        .expect("layer-shell is advertised");
    let shell = f.id();
    let mut args = word(&global.args).to_ne_bytes().to_vec();
    args.extend(string(interface));
    args.extend(4u32.to_ne_bytes());
    args.extend(shell.to_ne_bytes());
    f.wire.bytes(registry, 0, &args, None);
    f.dispatch();
    (f, shell)
}

pub(in crate::desktop::tests) fn map_window(f: &mut Fixture, buffer: u32) -> (Toplevel, Window) {
    let top = f.toplevel();
    f.configure(top);
    f.attach(top, buffer);
    let window = f
        .state
        .windows
        .iter()
        .find(|window| window.toplevel().unwrap().wl_surface().id().protocol_id() == top.surface)
        .unwrap()
        .clone();
    (top, window)
}
