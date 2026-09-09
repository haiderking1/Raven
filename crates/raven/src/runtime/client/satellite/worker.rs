use super::{process, reservation::Reservation};
use std::{
    ffi::OsStr,
    io::{self, Read},
    os::{fd::AsRawFd, unix::net::UnixStream},
    time::{Duration, Instant},
};

/// Poll sources live only here, never in the input/render event loop. The listener
/// entries are absent while satellite owns acceptance and during retry cooldown.
pub(super) fn run(
    binary: &OsStr,
    wayland: &OsStr,
    reservation: Reservation,
    mut control: UnixStream,
) -> io::Result<()> {
    let mut child: Option<process::Group> = None;
    let mut started = Instant::now();
    let mut rearm = None;
    let mut retry = Duration::from_secs(1);
    loop {
        if !reservation.intact() {
            return Err(io::Error::other(
                "X11 reservation paths changed; disabling satellite",
            ));
        }
        if let Some(group) = &child
            && group.exited()?
        {
            let status = child.take().unwrap().finish()?;
            eprintln!("raven: xwayland-satellite exited: {status}");
            if started.elapsed() >= Duration::from_secs(30) {
                retry = Duration::from_secs(1);
            }
            rearm = Some(Instant::now() + retry);
            retry = (retry * 2).min(Duration::from_secs(30));
        }
        if rearm.is_some_and(|deadline| Instant::now() >= deadline) {
            if reservation.discard_pending()? {
                rearm = None;
            } else {
                // A flood cannot turn queue clearing into an unbounded busy loop.
                rearm = Some(Instant::now() + Duration::from_secs(1));
            }
        }
        let armed = child.is_none() && rearm.is_none();
        let fds = reservation.fds();
        let mut sources = [
            source(control.as_raw_fd()),
            source(if armed { fds[0] } else { -1 }),
            source(if armed { fds[1] } else { -1 }),
            source(child.as_ref().map_or(-1, process::Group::poll_fd)),
        ];
        let timeout = rearm
            .map(|deadline| {
                deadline
                    .saturating_duration_since(Instant::now())
                    .as_millis()
                    .clamp(1, 1000) as i32
            })
            .unwrap_or(-1);
        // SAFETY: sources is a live array of pollfd values for this whole call.
        let result = unsafe { libc::poll(sources.as_mut_ptr(), sources.len() as _, timeout) };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if sources[0].revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
            return Ok(());
        }
        if sources[0].revents & libc::POLLIN != 0 {
            let mut bytes = [0; 64];
            match control.read(&mut bytes) {
                Ok(0) => return Ok(()),
                Ok(_) => (), // SIGCHLD notification; the next iteration checks our child.
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => (),
                Err(error) => return Err(error),
            }
        }
        if sources[1..3]
            .iter()
            .any(|fd| fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0)
        {
            return Err(io::Error::other("X11 activation listener failed"));
        }
        if armed
            && sources[1..3]
                .iter()
                .any(|fd| fd.revents & libc::POLLIN != 0)
        {
            if !reservation.intact() {
                return Err(io::Error::other("X11 reservation was replaced"));
            }
            // Do not accept the triggering connection. Xwayland consumes it from
            // the kernel backlog using the two inherited blocking listenfds.
            match process::launch(binary, wayland, reservation.display(), fds) {
                Ok(group) => {
                    child = Some(group);
                    started = Instant::now();
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
                    ) =>
                {
                    return Err(error);
                }
                Err(error) => {
                    eprintln!("raven: cannot start xwayland-satellite: {error}");
                    rearm = Some(Instant::now() + retry);
                    retry = (retry * 2).min(Duration::from_secs(30));
                }
            }
        }
    }
    // Local child drops before the reservation argument, killing its entire group
    // and waiting on this worker before descriptors and owned paths are released.
}

fn source(fd: i32) -> libc::pollfd {
    libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    }
}
