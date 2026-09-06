mod children;
mod spawn;

use children::Children;
use std::{error::Error, ffi::OsString};

/// Launch into Raven's actual socket, without changing the compositor's environment.
pub(crate) struct Clients {
    socket: OsString,
    children: Children,
}

impl Clients {
    pub(crate) fn new(socket: OsString) -> Self {
        Self {
            socket,
            children: Children::default(),
        }
    }

    pub(crate) fn spawn(&mut self, args: &[OsString]) -> Result<(), Box<dyn Error>> {
        if let Some(child) = spawn::client(args, &self.socket)? {
            self.children.track(child);
        }
        Ok(())
    }

    pub(crate) fn reap(&mut self) {
        self.children.reap();
    }
}
