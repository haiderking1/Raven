use super::super::{
    fixture::Toplevel,
    wire::{Event, word},
};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State;

pub(super) fn tiled(events: &[Event], top: Toplevel, size: (u32, u32)) -> u32 {
    let configs: Vec<_> = events
        .iter()
        .filter(|e| e.object == top.role && e.opcode == 0)
        .collect();
    assert_eq!(
        configs.len(),
        1,
        "one complete configure per response: {events:?}"
    );
    let config = configs[0];
    assert_eq!((word(&config.args), word(&config.args[4..])), size);
    let states: Vec<_> = config.args[12..].chunks_exact(4).map(word).collect();
    for edge in [
        State::TiledLeft,
        State::TiledRight,
        State::TiledTop,
        State::TiledBottom,
    ] {
        assert!(states.contains(&(edge as u32)));
    }
    assert!(!states.contains(&(State::Maximized as u32)));
    assert!(!states.contains(&(State::Fullscreen as u32)));
    let serials: Vec<_> = events
        .iter()
        .filter(|e| e.object == top.xdg && e.opcode == 0)
        .collect();
    assert_eq!(serials.len(), 1);
    word(&serials[0].args)
}
