use crate::{
    keyboard,
    runtime::settings::KeyboardSettings,
    wire::{Event, Wire, string, word},
};
use smithay::{
    delegate_compositor, delegate_seat,
    input::{SeatHandler, SeatState},
    reexports::wayland_server::{Display, backend::ClientData, protocol::wl_surface::WlSurface},
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
};
use std::{os::unix::net::UnixStream, sync::Arc};

struct State {
    seats: SeatState<Self>,
    compositor: CompositorState,
}
impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor
    }
    fn client_compositor_state<'a>(
        &self,
        client: &'a smithay::reexports::wayland_server::Client,
    ) -> &'a CompositorClientState {
        &client.get_data::<Client>().unwrap().0
    }
    fn commit(&mut self, _: &WlSurface) {}
}
delegate_compositor!(State);
impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seats
    }
}
delegate_seat!(State);
#[derive(Debug, Default)]
struct Client(CompositorClientState);
impl ClientData for Client {}

fn dispatch(display: &mut Display<State>, state: &mut State, wire: &mut Wire) -> Vec<Event> {
    display.dispatch_clients(state).unwrap();
    display.flush_clients().unwrap();
    wire.events()
}
fn repeat(events: &[Event], object: u32) -> Vec<(u32, u32)> {
    events
        .iter()
        .filter(|e| e.object == object && e.opcode == 5)
        .map(|e| (word(&e.args), word(&e.args[4..])))
        .collect()
}

#[test]
fn repeat_reload_updates_existing_and_future_keyboard_resources() {
    let mut display = Display::new().unwrap();
    let mut state = State {
        seats: SeatState::new(),
        compositor: CompositorState::new::<State>(&display.handle()),
    };
    let mut seat = state.seats.new_wl_seat(&display.handle(), "test-seat");
    seat.add_keyboard(Default::default(), 400, 25).unwrap();
    let (server, socket) = UnixStream::pair().unwrap();
    display
        .handle()
        .insert_client(server, Arc::new(Client::default()))
        .unwrap();
    let mut wire = Wire::new(socket);
    wire.request(1, 1, &[2]);
    let globals = dispatch(&mut display, &mut state, &mut wire);
    let global = globals
        .iter()
        .find(|e| {
            e.object == 2 && e.opcode == 0 && {
                let len = word(&e.args[4..]) as usize;
                &e.args[8..8 + len - 1] == b"wl_seat"
            }
        })
        .unwrap();
    let mut args = word(&global.args).to_ne_bytes().to_vec();
    args.extend(string("wl_seat"));
    args.extend(7u32.to_ne_bytes());
    args.extend(3u32.to_ne_bytes());
    wire.bytes(2, 0, &args, None);
    wire.request(3, 1, &[4]);
    assert_eq!(
        repeat(&dispatch(&mut display, &mut state, &mut wire), 4),
        [(25, 400)]
    );

    keyboard::apply(
        &seat,
        KeyboardSettings {
            repeat_rate: 0,
            repeat_delay: 250,
        },
    );
    assert_eq!(
        repeat(&dispatch(&mut display, &mut state, &mut wire), 4),
        [(0, 250)]
    );
    wire.request(3, 1, &[5]);
    assert_eq!(
        repeat(&dispatch(&mut display, &mut state, &mut wire), 5),
        [(0, 250)]
    );

    keyboard::apply(&seat, KeyboardSettings::default());
    let events = dispatch(&mut display, &mut state, &mut wire);
    assert_eq!(repeat(&events, 4), [(25, 400)]);
    assert_eq!(repeat(&events, 5), [(25, 400)]);
}
