mod children;
mod satellite;
mod spawn;

#[cfg(test)]
mod tests;

use children::Children;
use std::{error::Error, ffi::OsString};

/// Launch into Raven's actual socket, without changing the compositor's environment.
pub(crate) struct Clients {
    socket: OsString,
    children: Children,
    satellite: Option<satellite::Satellite>,
}

impl Clients {
    pub(crate) fn new(socket: OsString) -> Self {
        let satellite = satellite::Satellite::new(socket.clone());
        Self {
            socket,
            children: Children::default(),
            satellite,
        }
    }

    pub(crate) fn spawn(&mut self, args: &[OsString]) -> Result<(), Box<dyn Error>> {
        let display = self
            .satellite
            .as_ref()
            .and_then(satellite::Satellite::display);
        if let Some(child) = spawn::client(args, &self.socket, display)? {
            self.children.track(child);
        }
        Ok(())
    }

    pub(crate) fn reap(&mut self) {
        self.children.reap();
        if let Some(satellite) = &self.satellite {
            satellite.reap();
        }
    }
}
