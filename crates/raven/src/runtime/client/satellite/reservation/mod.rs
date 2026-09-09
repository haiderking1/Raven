mod directory;
mod listener;
mod path;

use directory::Directory;
use path::Entry;
use std::{
    io::{self, Write},
    os::unix::net::{SocketAddr, UnixListener},
    os::{
        fd::{AsRawFd, RawFd},
        linux::net::SocketAddrExt,
    },
};

/// No stale-file reclamation: an occupied lock, pathname or abstract name is skipped.
/// Field order closes listeners before removing the socket and finally its lock.
pub(super) struct Reservation {
    listeners: [UnixListener; 2],
    socket: Entry,
    lock: Entry,
    display: String,
}

impl Reservation {
    pub(super) fn acquire() -> io::Result<Self> {
        let temporary = Directory::temporary()?;
        let sockets = temporary.sockets()?;
        for number in 0..1000 {
            let mut lock = match Entry::create(temporary.clone(), &format!(".X{number}-lock")) {
                Ok(lock) => lock,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            };
            writeln!(lock.file(), "{:>10}", std::process::id())?;
            let name = format!("X{number}");
            let address = SocketAddr::from_abstract_name(format!("/tmp/.X11-unix/{name}"))?;
            let abstract_socket = match UnixListener::bind_addr(&address) {
                Ok(listener) => listener,
                Err(error) if error.kind() == io::ErrorKind::AddrInUse => continue,
                Err(error) => return Err(error),
            };
            let (filesystem_socket, socket) = match listener::filesystem(sockets.clone(), &name) {
                Ok(bound) => bound,
                Err(error) if error.kind() == io::ErrorKind::AddrInUse => continue,
                Err(error) => return Err(error),
            };
            let reservation = Self {
                listeners: [filesystem_socket, listener::above_stdio(abstract_socket)?],
                socket,
                lock,
                display: format!(":{number}"),
            };
            if !reservation.intact() {
                return Err(io::Error::other("X11 reservation was replaced"));
            }
            return Ok(reservation);
        }
        Err(io::Error::other(
            "no unoccupied X display in :0 through :999",
        ))
    }

    pub(super) fn display(&self) -> &str {
        &self.display
    }

    pub(super) fn fds(&self) -> [RawFd; 2] {
        self.listeners.each_ref().map(AsRawFd::as_raw_fd)
    }

    pub(super) fn intact(&self) -> bool {
        self.lock.visible() && self.socket.visible()
    }

    /// Called only with no server alive. Bound work under a connection flood.
    pub(super) fn discard_pending(&self) -> io::Result<bool> {
        let mut empty = true;
        for listener in &self.listeners {
            listener.set_nonblocking(true)?;
            let result = discard(listener);
            // Xwayland expects blocking listenfds. O_NONBLOCK is shared by dup/exec.
            listener.set_nonblocking(false)?;
            empty &= result?;
        }
        Ok(empty)
    }
}

fn discard(listener: &UnixListener) -> io::Result<bool> {
    for _ in 0..256 {
        match listener.accept() {
            Ok((connection, _)) => drop(connection),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(true),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::ConnectionAborted
                ) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(false)
}
