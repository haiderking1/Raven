use crate::state::{ClientState, State};
use smithay::{
    desktop::Window,
    reexports::wayland_server::{DisplayHandle, Resource},
};
use std::{
    io,
    os::{
        fd::AsRawFd,
        unix::{net::UnixStream, process::CommandExt},
    },
    process::Command,
    sync::Arc,
};

pub(super) fn attach(display: &mut DisplayHandle, command: &mut Command) -> io::Result<UnixStream> {
    let (server, child) = UnixStream::pair()?;
    display.insert_client(
        server,
        Arc::new(ClientState {
            configuration_error: true,
            ..Default::default()
        }),
    )?;
    let fd = child.as_raw_fd();
    command.env("WAYLAND_SOCKET", fd.to_string());
    // SAFETY: child is held through spawn; fcntl is async-signal-safe and only
    // changes this child's copy of the owned socket descriptor after fork.
    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok(child)
}

impl State {
    pub(crate) fn is_configuration_error(&self, window: &Window) -> bool {
        window
            .toplevel()
            .and_then(|top| top.wl_surface().client())
            .is_some_and(|client| {
                client
                    .get_data::<ClientState>()
                    .is_some_and(|data| data.configuration_error)
            })
    }
}
