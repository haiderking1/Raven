use crate::state::State;
use smithay::{
    input::pointer::*,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

pub(super) struct MoveGrab {
    pub(super) start: GrabStartData<State>,
}

// Gestures and scrolls during compositor movement are not client gestures.
macro_rules! suppress_gestures {
    ($($name:ident: $event:ty),* $(,)?) => {$(
        fn $name(&mut self, _data: &mut State, _handle: &mut PointerInnerHandle<'_, State>, _event: &$event) {}
    )*};
}

impl PointerGrab<State> for MoveGrab {
    fn motion(
        &mut self,
        data: &mut State,
        handle: &mut PointerInnerHandle<'_, State>,
        _focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &MotionEvent,
    ) {
        let Some(mut drag) = data.input.drag.take() else {
            handle.unset_grab(self, data, event.serial, event.time, true);
            return;
        };
        if !drag.valid(data) {
            drag.queue_resize_cleanup(data);
            handle.motion(data, None, event);
            handle.unset_grab(self, data, event.serial, event.time, true);
            data.request_redraw();
            return;
        }
        if let Some(resize) = drag.resize.as_mut() {
            resize.motion(event.location);
        } else if drag.floating {
            let offset = (event.location - drag.start).to_i32_round();
            data.move_floating_window(&drag.window, drag.frame.loc + offset);
        } else if data.pointer_location != event.location {
            data.request_redraw();
        }
        data.input.drag = Some(drag);
        handle.motion(data, None, event);
    }

    fn relative_motion(
        &mut self,
        data: &mut State,
        handle: &mut PointerInnerHandle<'_, State>,
        _focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &RelativeMotionEvent,
    ) {
        handle.relative_motion(data, None, event);
    }

    // Hardware button pairs are consumed by dragging::button before dispatch.
    fn button(
        &mut self,
        _data: &mut State,
        _handle: &mut PointerInnerHandle<'_, State>,
        _event: &ButtonEvent,
    ) {
    }
    fn axis(
        &mut self,
        _data: &mut State,
        _handle: &mut PointerInnerHandle<'_, State>,
        _details: AxisFrame,
    ) {
    }
    fn frame(&mut self, data: &mut State, handle: &mut PointerInnerHandle<'_, State>) {
        handle.frame(data);
    }
    suppress_gestures! {
        gesture_swipe_begin: GestureSwipeBeginEvent,
        gesture_swipe_update: GestureSwipeUpdateEvent,
        gesture_swipe_end: GestureSwipeEndEvent,
        gesture_pinch_begin: GesturePinchBeginEvent,
        gesture_pinch_update: GesturePinchUpdateEvent,
        gesture_pinch_end: GesturePinchEndEvent,
        gesture_hold_begin: GestureHoldBeginEvent,
        gesture_hold_end: GestureHoldEndEvent,
    }
    fn start_data(&self) -> &GrabStartData<State> {
        &self.start
    }
    fn unset(&mut self, data: &mut State) {
        if let Some(drag) = data.input.drag.take() {
            drag.queue_resize_cleanup(data);
            data.request_redraw();
        }
    }
}
