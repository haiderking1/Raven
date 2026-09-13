use crate::server::State;
use smithay::{
    delegate_compositor, delegate_output, delegate_seat,
    input::{SeatHandler, SeatState},
    reexports::wayland_server::{
        Client as WaylandClient, backend::ClientData, protocol::wl_surface::WlSurface,
    },
    wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        output::OutputHandler,
    },
};
#[derive(Debug, Default)]
pub struct Client(CompositorClientState);
impl ClientData for Client {}
impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor
    }
    fn client_compositor_state<'a>(&self, client: &'a WaylandClient) -> &'a CompositorClientState {
        &client.get_data::<Client>().unwrap().0
    }
    fn commit(&mut self, _: &WlSurface) {}
}
impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seats
    }
}
impl OutputHandler for State {}
delegate_compositor!(State);
delegate_seat!(State);
delegate_output!(State);
