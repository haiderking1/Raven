use super::Observation;
use smithay::reexports::x11rb::{
    COPY_DEPTH_FROM_PARENT,
    connection::Connection,
    protocol::xproto::{
        Atom, AtomEnum, ClientMessageEvent, ConnectionExt, CreateWindowAux, EventMask, PropMode,
        WindowClass,
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};
use std::error::Error;

pub(super) struct Window {
    pub id: u32,
    root: u32,
    state: Atom,
    fullscreen: Atom,
}

impl Window {
    pub fn map(connection: &RustConnection) -> Result<Self, Box<dyn Error>> {
        let screen = &connection.setup().roots[0];
        let id = connection.generate_id()?;
        let state = connection
            .intern_atom(false, b"_NET_WM_STATE")?
            .reply()?
            .atom;
        let fullscreen = connection
            .intern_atom(false, b"_NET_WM_STATE_FULLSCREEN")?
            .reply()?
            .atom;
        connection
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                id,
                screen.root,
                0,
                0,
                320,
                200,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &CreateWindowAux::new()
                    .background_pixel(0x306090)
                    .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY),
            )?
            .check()?;
        connection
            .change_property8(
                PropMode::REPLACE,
                id,
                AtomEnum::WM_NAME,
                AtomEnum::STRING,
                b"raven-satellite-integration",
            )?
            .check()?;
        connection.map_window(id)?.check()?;
        connection.clear_area(false, id, 0, 0, 0, 0)?.check()?;
        connection.flush()?;
        Ok(Self {
            id,
            root: screen.root,
            state,
            fullscreen,
        })
    }

    pub fn fullscreen(&self, connection: &RustConnection) -> Result<(), Box<dyn Error>> {
        let event = ClientMessageEvent::new(32, self.id, self.state, [1, self.fullscreen, 0, 1, 0]);
        connection
            .send_event(
                false,
                self.root,
                EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                event,
            )?
            .check()?;
        connection.flush()?;
        Ok(())
    }

    pub fn inspect(&self, connection: &RustConnection) -> Result<Observation, Box<dyn Error>> {
        let geometry = connection.get_geometry(self.id)?.reply()?;
        let property = connection
            .get_property(false, self.id, self.state, AtomEnum::ATOM, 0, 32)?
            .reply()?;
        let fullscreen = property
            .value32()
            .is_some_and(|mut atoms| atoms.any(|atom| atom == self.fullscreen));
        Ok(Observation::Geometry {
            width: geometry.width,
            height: geometry.height,
            fullscreen,
        })
    }
}
