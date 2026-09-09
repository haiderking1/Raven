use super::super::{
    fixture::{Fixture, Toplevel},
    wire::{Event, word},
};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

pub(super) fn no_configure(events: &[Event], top: Toplevel) {
    assert!(
        !events
            .iter()
            .any(|event| event.object == top.role || event.object == top.xdg),
        "no role events before the first bufferless commit: {events:?}"
    );
}

pub(super) fn configured(
    events: &[Event],
    top: Toplevel,
    size: (u32, u32),
    fullscreen: bool,
) -> u32 {
    let configs: Vec<_> = events
        .iter()
        .filter(|e| e.object == top.role && e.opcode == 0)
        .collect();
    assert_eq!(configs.len(), 1, "one complete configure: {events:?}");
    let config = configs[0];
    assert_eq!((word(&config.args), word(&config.args[4..])), size);
    let states: Vec<_> = config.args[12..].chunks_exact(4).map(word).collect();
    assert_eq!(states.contains(&(State::Fullscreen as u32)), fullscreen);
    assert!(!states.contains(&(State::Maximized as u32)));
    let serials: Vec<_> = events
        .iter()
        .filter(|e| e.object == top.xdg && e.opcode == 0)
        .collect();
    assert_eq!(serials.len(), 1, "each state configure has a serial");
    word(&serials[0].args)
}

// Mapping can configure the displaced owner's size, then its activation state.
pub(super) fn latest_configured(
    events: &[Event],
    top: Toplevel,
    size: (u32, u32),
    fullscreen: bool,
) -> u32 {
    let mut start = 0;
    let mut latest = None;
    for (end, event) in events.iter().enumerate() {
        if event.object == top.xdg && event.opcode == 0 {
            latest = Some(configured(&events[start..=end], top, size, fullscreen));
            start = end + 1;
        }
    }
    assert!(
        !events[start..]
            .iter()
            .any(|event| event.object == top.role && event.opcode == 0),
        "every role configure must have a serial"
    );
    latest.expect("at least one complete configure")
}

pub(super) fn owner(f: &Fixture, expected: Option<&Window>) {
    assert_eq!(f.state.fullscreen_window(), expected);
}

pub(super) fn layout(f: &Fixture, window: &Window, loc: (i32, i32), size: (i32, i32)) {
    assert_eq!(
        f.state.window_layout_geometry(window),
        Some(Rectangle::<i32, Logical>::new(loc.into(), size.into()))
    );
}

pub(super) fn snapshot(f: &Fixture, windows: &[&Window]) -> Vec<Rectangle<i32, Logical>> {
    windows
        .iter()
        .map(|window| f.state.window_layout_geometry(window).unwrap())
        .collect()
}
