use super::{directory::Directory, path::Entry};
use std::{
    io,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::UnixListener,
    },
    sync::Arc,
};

pub(super) fn filesystem(
    directory: Arc<Directory>,
    name: &str,
) -> io::Result<(UnixListener, Entry)> {
    let path = directory.bind_path(name);
    // SAFETY: zero is a valid initial representation for sockaddr_un.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    address.sun_family = libc::AF_UNIX as _;
    if path.len() >= address.sun_path.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "X11 socket path is too long",
        ));
    }
    for (slot, byte) in address.sun_path.iter_mut().zip(path.bytes()) {
        *slot = byte as _;
    }
    // SAFETY: socket returns a fresh owned descriptor, checked before conversion.
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let listener = above_stdio(unsafe { UnixListener::from_raw_fd(fd) })?;
    let length = std::mem::offset_of!(libc::sockaddr_un, sun_path) + path.len() + 1;
    // SAFETY: address contains a terminated path and length includes its NUL byte.
    if unsafe {
        libc::bind(
            listener.as_raw_fd(),
            (&address as *const libc::sockaddr_un).cast(),
            length as _,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    // Register ownership before listen, so even a listen failure removes our path.
    let entry = Entry::socket(directory, name)?;
    if unsafe { libc::listen(listener.as_raw_fd(), 128) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((listener, entry))
}

pub(super) fn above_stdio(listener: UnixListener) -> io::Result<UnixListener> {
    // Command's stdio setup must not overwrite a listenfd when Raven inherits
    // closed standard descriptors. Only the exec child will clear CLOEXEC.
    if listener.as_raw_fd() >= 3 {
        return Ok(listener);
    }
    let fd = unsafe { libc::fcntl(listener.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { UnixListener::from_raw_fd(fd) })
}
