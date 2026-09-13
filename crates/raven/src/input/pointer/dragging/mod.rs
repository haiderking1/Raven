mod admission;
mod grab;
mod lifecycle;
mod resizing;
mod targeting;

use crate::state::State;
use smithay::{
    backend::input::ButtonState,
    desktop::Window,
    output::Output,
    utils::{Logical, Point, Rectangle, Serial, Transform},
};

const MOVE_BUTTON: u32 = 0x110;
const RESIZE_BUTTON: u32 = 0x111;

#[derive(Debug)]
pub(in crate::input) struct Drag {
    window: Window,
    button: u32,
    resize: Option<resizing::Resize>,
    start: Point<f64, Logical>,
    frame: Rectangle<i32, Logical>,
    floating: bool,
    workspace: usize,
    output: Output,
    area: Rectangle<i32, Logical>,
    scale: f64,
    transform: Transform,
    serial: Serial,
}

/// Compositor-owned button pairs never enter client implicit-click state.
/// Keep consuming releases after cancellation or replacement by another grab.
pub(super) fn button(
    state: &mut State,
    button: u32,
    pressed: ButtonState,
    serial: Serial,
    time: u32,
) -> bool {
    state.reconcile_window_drag();
    if pressed == ButtonState::Released && state.input.drag_buttons.remove(&button) {
        let finishing = state
            .input
            .drag
            .as_ref()
            .is_some_and(|drag| drag.button == button);
        if finishing {
            state.reset_floating_resize_pacing();
        }
        state.apply_window_resize();
        if finishing {
            state.finish_window_drag(serial, time);
        }
        return true;
    }
    if state.input.drag.is_some() {
        if pressed == ButtonState::Pressed {
            state.input.drag_buttons.insert(button);
        }
        return true;
    }
    if pressed == ButtonState::Pressed
        && matches!(button, MOVE_BUTTON | RESIZE_BUTTON)
        && state.input.drag_buttons.is_empty()
        && admission::begin(state, button, serial)
    {
        state.input.drag_buttons.insert(button);
        return true;
    }
    false
}
