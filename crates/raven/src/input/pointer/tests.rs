use super::motion;
use crate::state::State;
use smithay::reexports::{calloop::EventLoop, wayland_server::Display};

#[test]
fn pointer_movement_requests_redraw_but_stationary_input_does_not() {
    let display = Display::<State>::new().unwrap();
    let event_loop = EventLoop::<State>::try_new().unwrap();
    let mut state = State::new(display.handle(), event_loop.get_signal()).unwrap();
    motion(&mut state, (10.0, 20.0).into(), 1, None);
    assert!(state.take_redraw_request());
    motion(&mut state, (10.0, 20.0).into(), 2, None);
    assert!(!state.take_redraw_request());
    motion(&mut state, (11.0, 20.0).into(), 3, None);
    assert!(state.take_redraw_request());
}
