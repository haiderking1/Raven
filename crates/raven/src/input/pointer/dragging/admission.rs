use super::{
    Drag, RESIZE_BUTTON,
    grab::MoveGrab,
    resizing::{self, Resize},
};
use crate::state::State;
use smithay::{
    input::pointer::{Focus, GrabStartData},
    utils::Serial,
    wayland::shell::wlr_layer::Layer,
};

pub(super) fn begin(state: &mut State, button: u32, serial: Serial) -> bool {
    let Some(pointer) = state.seat.get_pointer() else {
        return false;
    };
    let Some(keyboard) = state.seat.get_keyboard() else {
        return false;
    };
    let modifiers = keyboard.modifier_state();
    if !modifiers.logo
        || modifiers.ctrl
        || modifiers.alt
        || modifiers.shift
        || pointer.is_grabbed()
        || keyboard.is_grabbed()
        || state.pointer_is_captured()
        || state.exclusive_keyboard_layer().is_some()
        || state
            .layer_under(state.pointer_location, &[Layer::Overlay, Layer::Top])
            .is_some()
    {
        return false;
    }
    let Some(window) = state.window_focus_under(state.pointer_location).cloned() else {
        return false;
    };
    if state.fullscreen_manages(&window) {
        return false;
    }
    let floating = state.window_is_floating(&window);
    if !floating && !state.window_is_tiled(&window) {
        return false;
    }
    let Some(output) = state.output.clone() else {
        return false;
    };
    let Some(area) = state.space().output_geometry(&output) else {
        return false;
    };
    state.activate_window(Some(window.clone()));
    state.cancel_resize_window(&window);
    if let Some(top) = window.toplevel() {
        if let Some(backend) = state.backend.as_mut() {
            backend.cancel_resize_surface(top.wl_surface());
        }
    }
    let Some(frame) = state.window_frame_geometry(&window) else {
        return false;
    };
    let resize = if button == RESIZE_BUTTON {
        let Some(resize) = Resize::new(state, &window, frame, floating) else {
            return false;
        };
        Some(resize)
    } else {
        None
    };
    if let Some(tiles) = resize.as_ref().and_then(|resize| resize.tiles.as_ref()) {
        if let Some(backend) = state.backend.as_mut() {
            for affected in state.windows.iter().filter(|window| tiles.contains(window)) {
                if let Some(top) = affected.toplevel() {
                    backend.cancel_resize_surface(top.wl_surface());
                }
            }
        }
    }
    state.input.drag = Some(Drag {
        window: window.clone(),
        button,
        resize,
        start: state.pointer_location,
        frame,
        floating,
        workspace: state.workspaces.active,
        area,
        scale: output.current_scale().fractional_scale(),
        transform: output.current_transform(),
        output,
        serial,
    });
    if button == RESIZE_BUTTON {
        let participants: Vec<_> = state
            .windows
            .iter()
            .filter(|window| {
                window
                    .toplevel()
                    .is_some_and(|top| state.surface_is_dragged_window(top.wl_surface()))
            })
            .cloned()
            .collect();
        for participant in participants {
            state.begin_live_resize(&participant);
        }
        state.restack_floating(state.workspaces.active);
    }
    let start = GrabStartData {
        focus: None,
        button,
        location: state.pointer_location,
    };
    pointer.set_grab(state, MoveGrab { start }, serial, Focus::Clear);
    if button == RESIZE_BUTTON {
        resizing::set_resizing(state, &window, true);
    }
    state.request_redraw();
    true
}
