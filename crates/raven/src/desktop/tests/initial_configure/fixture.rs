use super::super::fixture::Fixture;
use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};

pub(super) fn fixture(occupied: bool) -> Fixture {
    let mut f = Fixture::new();
    let output = Output::new(
        "startup-test".into(),
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
    if occupied {
        let top = f.toplevel();
        f.configure(top);
        let buffer = f.buffer_sized(1001, 601);
        f.attach(top, buffer);
    }
    f
}
