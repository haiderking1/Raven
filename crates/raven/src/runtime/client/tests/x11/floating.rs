use super::Observation;
use smithay::reexports::x11rb::{
    COPY_DEPTH_FROM_PARENT,
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ConnectionExt, CreateWindowAux, EventMask, MapState, PropMode, WindowClass,
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};
use std::error::Error;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Kind {
    Transient,
    Splash,
}

impl Kind {
    pub fn index(self) -> usize {
        match self {
            Self::Transient => 0,
            Self::Splash => 1,
        }
    }

    pub fn size(self) -> (u16, u16) {
        match self {
            Self::Transient => (300, 180),
            Self::Splash => (160, 100),
        }
    }
}

#[derive(Default)]
pub(super) struct Windows {
    ids: [Option<u32>; 2],
}

impl Windows {
    pub fn contains(&self, id: u32) -> bool {
        self.ids.contains(&Some(id))
    }

    pub fn map(
        &mut self,
        connection: &RustConnection,
        parent: u32,
        kind: Kind,
    ) -> Result<(), Box<dyn Error>> {
        if self.ids[kind.index()].is_some() {
            return Err("floating X11 window already exists".into());
        }
        let screen = &connection.setup().roots[0];
        let id = connection.generate_id()?;
        let (width, height) = kind.size();
        connection
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                id,
                screen.root,
                0,
                0,
                width,
                height,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &CreateWindowAux::new()
                    .background_pixel(0x906030)
                    .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY),
            )?
            .check()?;
        match kind {
            Kind::Transient => {
                connection
                    .change_property32(
                        PropMode::REPLACE,
                        id,
                        AtomEnum::WM_TRANSIENT_FOR,
                        AtomEnum::WINDOW,
                        &[parent],
                    )?
                    .check()?;
            }
            Kind::Splash => {
                let property = connection
                    .intern_atom(false, b"_NET_WM_WINDOW_TYPE")?
                    .reply()?
                    .atom;
                let splash = connection
                    .intern_atom(false, b"_NET_WM_WINDOW_TYPE_SPLASH")?
                    .reply()?
                    .atom;
                connection
                    .change_property32(PropMode::REPLACE, id, property, AtomEnum::ATOM, &[splash])?
                    .check()?;
                // No transient parent or WM_NORMAL_HINTS: Satellite must translate
                // the splash type into fixed XDG min/max sizes itself.
            }
        }
        connection.map_window(id)?.check()?;
        connection.clear_area(false, id, 0, 0, 0, 0)?.check()?;
        connection.flush()?;
        self.ids[kind.index()] = Some(id);
        Ok(())
    }

    pub fn inspect(
        &self,
        connection: &RustConnection,
        kind: Kind,
    ) -> Result<Observation, Box<dyn Error>> {
        let id = self.ids[kind.index()].ok_or("floating X11 window missing")?;
        let geometry = connection.get_geometry(id)?.reply()?;
        let attributes = connection.get_window_attributes(id)?.reply()?;
        Ok(Observation::FloatingGeometry {
            kind,
            width: geometry.width,
            height: geometry.height,
            mapped: attributes.map_state == MapState::VIEWABLE,
        })
    }

    pub fn close(&mut self, connection: &RustConnection, kind: Kind) -> Result<(), Box<dyn Error>> {
        if let Some(id) = self.ids[kind.index()].take() {
            connection.destroy_window(id)?.check()?;
            connection.flush()?;
        }
        Ok(())
    }
}
