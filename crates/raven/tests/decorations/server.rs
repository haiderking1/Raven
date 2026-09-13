use crate::{
    kde::KdeDecorations,
    wire::{Event, Wire, string, word},
};
use smithay::{
    delegate_compositor,
    reexports::wayland_server::{
        Client, Display, backend::ClientData, protocol::wl_surface::WlSurface,
    },
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
};
use std::{os::unix::net::UnixStream, sync::Arc};

pub struct State {
    compositor: CompositorState,
    _decorations: KdeDecorations,
}

#[derive(Debug, Default)]
struct ClientState(CompositorClientState);
impl ClientData for ClientState {}

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor
    }
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().0
    }
    fn commit(&mut self, _: &WlSurface) {}
}
delegate_compositor!(State);

pub struct Server {
    state: State,
    display: Display<State>,
    pub wire: Wire,
}

impl Server {
    pub fn new() -> Self {
        let display = Display::new().unwrap();
        let handle = display.handle();
        let state = State {
            compositor: CompositorState::new::<State>(&handle),
            _decorations: KdeDecorations::new(&handle),
        };
        let (server, client) = UnixStream::pair().unwrap();
        display
            .handle()
            .insert_client(server, Arc::new(ClientState::default()))
            .unwrap();
        let mut server = Self {
            state,
            display,
            wire: Wire::new(client),
        };
        server.wire.request(1, 1, &[2]);
        let globals = server.dispatch();
        for (name, id) in [
            ("wl_compositor", 3),
            ("org_kde_kwin_server_decoration_manager", 4),
        ] {
            let global = globals
                .iter()
                .find(|e| {
                    e.object == 2 && e.opcode == 0 && {
                        let len = word(&e.args[4..]) as usize;
                        &e.args[8..8 + len - 1] == name.as_bytes()
                    }
                })
                .expect("required global advertised");
            let mut args = word(&global.args).to_ne_bytes().to_vec();
            args.extend(string(name));
            args.extend(1u32.to_ne_bytes());
            args.extend(u32::to_ne_bytes(id));
            server.wire.bytes(2, 0, &args, None);
        }
        server
    }

    pub fn dispatch(&mut self) -> Vec<Event> {
        self.display.dispatch_clients(&mut self.state).unwrap();
        self.display.flush_clients().unwrap();
        self.wire.events()
    }
}
