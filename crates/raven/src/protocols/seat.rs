use crate::state::State;
use smithay::{
    delegate_seat,
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    wayland::selection::data_device::set_data_device_focus,
};

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        set_data_device_focus(
            &self.display_handle,
            seat,
            focused.and_then(Resource::client),
        );
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor_status = image;
    }
}
delegate_seat!(State);
