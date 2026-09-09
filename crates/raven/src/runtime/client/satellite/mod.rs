//! On-demand X11 service owned by Clients. No process-global environment changes.
//! RAVEN_XWAYLAND_SATELLITE=off disables it; any other value selects the executable.
//! Unset uses xwayland-satellite from PATH. Only managed clients receive DISPLAY.
//! Probe failure leaves native Wayland enabled. Setup precedes backend installation;
//! teardown joins the worker only after the backend has been dropped.
mod process;
mod reservation;
mod worker;

use std::{
    ffi::{OsStr, OsString},
    io::{self, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};

pub(super) struct Satellite {
    display: String,
    available: Arc<AtomicBool>,
    control: UnixStream,
    worker: Option<JoinHandle<()>>,
}

impl Satellite {
    /// Startup only, before installing the backend. The bounded probe completes
    /// before the first managed client receives DISPLAY.
    pub(super) fn new(wayland: OsString) -> Option<Self> {
        let binary = std::env::var_os("RAVEN_XWAYLAND_SATELLITE")
            .unwrap_or_else(|| OsString::from("xwayland-satellite"));
        if binary == "off" {
            return None;
        }
        match Self::start(binary, wayland) {
            Ok(satellite) => Some(satellite),
            Err(error) => {
                eprintln!("raven: X11 disabled, native Wayland remains available: {error}");
                None
            }
        }
    }

    fn start(binary: OsString, wayland: OsString) -> io::Result<Self> {
        let (control, receiver) = UnixStream::pair()?;
        control.set_nonblocking(true)?;
        receiver.set_nonblocking(true)?;
        let available = Arc::new(AtomicBool::new(false));
        let status = available.clone();
        let (ready, result) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("raven-x11".into())
            .spawn(move || {
                let startup = reservation::Reservation::acquire().and_then(|reservation| {
                    process::supported(&binary, reservation.display())?;
                    Ok(reservation)
                });
                let reservation = match startup {
                    Ok(reservation) => reservation,
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                status.store(true, Ordering::Release);
                let _availability = Availability(status);
                if ready.send(Ok(reservation.display().to_owned())).is_err() {
                    return;
                }
                if let Err(error) = worker::run(&binary, &wayland, reservation, receiver) {
                    eprintln!("raven: X11 disabled, native Wayland remains available: {error}");
                }
            })?;
        match result.recv() {
            Ok(Ok(display)) => {
                eprintln!("raven: reserved {display} for on-demand xwayland-satellite");
                Ok(Self {
                    display,
                    available,
                    control,
                    worker: Some(worker),
                })
            }
            result => {
                let _ = worker.join();
                Err(match result {
                    Ok(Err(error)) => error,
                    _ => io::Error::other("satellite worker stopped during startup"),
                })
            }
        }
    }

    pub(super) fn display(&self) -> Option<&OsStr> {
        self.available
            .load(Ordering::Acquire)
            .then(|| OsStr::new(&self.display))
    }

    pub(super) fn reap(&self) {
        // SIGCHLD handling never waits or reaps the worker's child. A full channel
        // already contains a wakeup; the child pidfd independently reports exit.
        let _ = (&self.control).write(&[1]);
    }
}

impl Drop for Satellite {
    fn drop(&mut self) {
        // Clients are dropped after backend teardown, never from a dispatch callback.
        // EOF is reliable even if the nonblocking wakeup channel is full.
        let _ = self.control.shutdown(Shutdown::Both);
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            eprintln!("raven: satellite worker panicked");
        }
    }
}

struct Availability(Arc<AtomicBool>);
impl Drop for Availability {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
