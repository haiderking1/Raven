use super::super::{
    fixture::{Fixture, Toplevel},
    wire::word,
};

pub(in crate::desktop::tests) struct Popup {
    pub surface: u32,
    pub xdg: u32,
    pub role: u32,
    pub positioner: u32,
}

pub(in crate::desktop::tests) fn map_popup(f: &mut Fixture, parent: Toplevel) -> Popup {
    map_positioned_popup(f, parent, false, [0, 0, 80, 70])
}

pub(in crate::desktop::tests) fn map_reactive_popup(
    f: &mut Fixture,
    parent: Toplevel,
    anchor: [u32; 4],
) -> Popup {
    map_positioned_popup(f, parent, true, anchor)
}

fn map_positioned_popup(
    f: &mut Fixture,
    parent: Toplevel,
    reactive: bool,
    anchor: [u32; 4],
) -> Popup {
    let popup = Popup {
        positioner: f.id(),
        surface: f.id(),
        xdg: f.id(),
        role: f.id(),
    };
    f.wire.request(5, 1, &[popup.positioner]);
    f.wire.request(popup.positioner, 1, &[30, 20]);
    f.wire.request(popup.positioner, 2, &anchor);
    if reactive {
        f.wire.request(popup.positioner, 5, &[3]); // slide X and Y
        f.wire.request(popup.positioner, 7, &[]);
    }
    f.wire.request(3, 0, &[popup.surface]);
    f.wire.request(5, 2, &[popup.xdg, popup.surface]);
    f.wire
        .request(popup.xdg, 2, &[popup.role, parent.xdg, popup.positioner]);
    assert!(
        !f.dispatch()
            .iter()
            .any(|e| e.object == popup.xdg && e.opcode == 0)
    );
    f.wire.request(popup.surface, 6, &[]);
    let events = f.dispatch();
    assert!(
        events
            .iter()
            .any(|e| e.object == popup.role && e.opcode == 0)
    );
    let configure = events
        .iter()
        .find(|e| e.object == popup.xdg && e.opcode == 0)
        .unwrap();
    f.wire.request(popup.xdg, 4, &[word(&configure.args)]);
    let buffer = f.buffer_sized(30, 20);
    f.wire.request(popup.surface, 1, &[buffer, 0, 0]);
    f.wire.request(popup.surface, 6, &[]);
    f.dispatch();
    popup
}
