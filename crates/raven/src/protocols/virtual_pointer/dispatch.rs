use super::{Event, Handler, PointerData, VirtualPointerState, frame};
use smithay::reexports::{
    wayland_protocols_wlr::virtual_pointer::v1::server::zwlr_virtual_pointer_v1::{
        Error, Request, ZwlrVirtualPointerV1,
    },
    wayland_server::{
        Client, DataInit, Dispatch, DisplayHandle, Resource, WEnum, backend::ClientId,
        protocol::wl_pointer::ButtonState,
    },
};

impl<D: Handler> Dispatch<ZwlrVirtualPointerV1, PointerData<D>, D> for VirtualPointerState {
    fn request(
        state: &mut D,
        _: &Client,
        pointer: &ZwlrVirtualPointerV1,
        request: Request,
        data: &PointerData<D>,
        _: &DisplayHandle,
        _: &mut DataInit<'_, D>,
    ) {
        let mut pending = data.pending.lock().unwrap();
        if pending.epoch != state.pointer_epoch() {
            pending.reset(state.pointer_epoch());
        }
        let active = state.pointer_active(&data.mapping);
        if !active {
            pending.reset(state.pointer_epoch());
        }
        let event = match request {
            Request::Motion { time, dx, dy } => Event::Motion { time, dx, dy },
            Request::MotionAbsolute {
                time,
                x,
                y,
                x_extent,
                y_extent,
            } => {
                if x_extent == 0 || y_extent == 0 {
                    return;
                }
                Event::Absolute {
                    time,
                    x,
                    y,
                    x_extent,
                    y_extent,
                }
            }
            Request::Button {
                time,
                button,
                state,
            } => {
                let pressed = match state {
                    WEnum::Value(ButtonState::Pressed) => true,
                    WEnum::Value(ButtonState::Released) => false,
                    _ => return,
                };
                Event::Button {
                    time,
                    button,
                    pressed,
                }
            }
            Request::Axis { time, axis, value }
            | Request::AxisDiscrete {
                time, axis, value, ..
            } => {
                let Some(axis) = frame::axis(axis) else {
                    pointer.post_error(Error::InvalidAxis, "invalid virtual pointer axis");
                    return;
                };
                let discrete = if let Request::AxisDiscrete { discrete, .. } = request {
                    Some(discrete)
                } else {
                    None
                };
                if active {
                    pending.axis(time, axis, value, discrete, false);
                }
                return;
            }
            Request::AxisStop { time, axis } => {
                let Some(axis) = frame::axis(axis) else {
                    pointer.post_error(Error::InvalidAxis, "invalid virtual pointer axis");
                    return;
                };
                if active {
                    pending.axis(time, axis, 0.0, None, true);
                }
                return;
            }
            Request::AxisSource { axis_source } => {
                let Some(source) = frame::source(axis_source) else {
                    pointer.post_error(
                        Error::InvalidAxisSource,
                        "invalid virtual pointer axis source",
                    );
                    return;
                };
                if active {
                    pending.source = Some(source);
                }
                return;
            }
            Request::Frame => Event::Frame(pending.take()),
            Request::Destroy => return,
            _ => unreachable!(),
        };
        drop(pending);
        if active {
            state.pointer_event(pointer, &data.mapping, event);
        }
    }

    fn destroyed(state: &mut D, _: ClientId, pointer: &ZwlrVirtualPointerV1, _: &PointerData<D>) {
        state.pointer_destroyed(pointer.id());
    }
}
