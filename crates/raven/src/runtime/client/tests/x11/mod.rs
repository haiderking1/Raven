mod floating;
mod window;

pub(super) use floating::Kind as FloatingKind;

use smithay::reexports::x11rb::{
    connection::Connection,
    protocol::{Event, xproto::ConnectionExt},
    rust_connection::{DefaultStream, RustConnection},
};
use std::{
    error::Error,
    os::unix::net::UnixStream,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub(super) enum Command {
    Fullscreen,
    Inspect,
    OpenFloating(FloatingKind),
    InspectFloating(FloatingKind),
    CloseFloating(FloatingKind),
    Stop,
}
pub(super) enum Observation {
    Mapped,
    FloatingGeometry {
        kind: FloatingKind,
        width: u16,
        height: u16,
        mapped: bool,
    },
    Geometry {
        width: u16,
        height: u16,
        fullscreen: bool,
    },
}

pub(super) struct Client {
    commands: Sender<Command>,
    observations: Receiver<Result<Observation, String>>,
    thread: JoinHandle<Result<(), String>>,
    pub mapped: bool,
    pub geometry: Option<(u16, u16, bool)>,
    pub floating_geometry: [Option<(u16, u16, bool)>; 2],
}

impl Client {
    pub fn start(display: &str, deadline: Instant) -> Result<Self, Box<dyn Error>> {
        let number: u16 = display
            .strip_prefix(':')
            .ok_or("not a reserved local DISPLAY")?
            .parse()?;
        let path = format!("/tmp/.X11-unix/X{number}");
        let (commands, receive) = mpsc::channel();
        let (send, observations) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("satellite-regression-x11".into())
            .spawn(move || {
                let result =
                    run(&path, deadline, receive, &send).map_err(|error| error.to_string());
                if let Err(error) = &result {
                    let _ = send.send(Err(error.clone()));
                }
                result
            })?;
        Ok(Self {
            commands,
            observations,
            thread,
            mapped: false,
            geometry: None,
            floating_geometry: [None; 2],
        })
    }

    pub fn command(&self, command: Command) -> Result<(), Box<dyn Error>> {
        self.commands.send(command)?;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<(), Box<dyn Error>> {
        while let Ok(observation) = self.observations.try_recv() {
            match observation? {
                Observation::Mapped => self.mapped = true,
                Observation::FloatingGeometry {
                    kind,
                    width,
                    height,
                    mapped,
                } => {
                    self.floating_geometry[kind.index()] = Some((width, height, mapped));
                }
                Observation::Geometry {
                    width,
                    height,
                    fullscreen,
                } => self.geometry = Some((width, height, fullscreen)),
            }
        }
        Ok(())
    }

    pub fn finished(&self) -> bool {
        self.thread.is_finished()
    }

    pub fn join(self) -> Result<(), Box<dyn Error>> {
        self.thread
            .join()
            .map_err(|_| "X11 client thread panicked")??;
        Ok(())
    }
}

fn run(
    path: &str,
    deadline: Instant,
    commands: Receiver<Command>,
    observations: &Sender<Result<Observation, String>>,
) -> Result<(), Box<dyn Error>> {
    // Connect ONLY to the production reservation, with no DISPLAY fallback or xauth reads.
    let (stream, _) = DefaultStream::from_unix_stream(UnixStream::connect(path)?)?;
    let connection = RustConnection::connect_to_stream(stream, 0)?;
    let window = window::Window::map(&connection)?;
    let mut floating = floating::Windows::default();
    observations.send(Ok(Observation::Mapped))?;
    loop {
        while let Some(event) = connection.poll_for_event()? {
            let repaint = match &event {
                Event::Expose(event) => Some(event.window),
                Event::ConfigureNotify(event) => Some(event.window),
                _ => None,
            };
            match event {
                Event::Expose(_) | Event::ConfigureNotify(_)
                    if repaint.is_some_and(|id| id == window.id || floating.contains(id)) =>
                {
                    connection
                        .clear_area(false, repaint.unwrap(), 0, 0, 0, 0)?
                        .check()?;
                    connection.flush()?;
                }
                Event::Error(error) => return Err(format!("X11 protocol error: {error:?}").into()),
                _ => (),
            }
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or("X11 client deadline exceeded")?;
        match commands.recv_timeout(remaining.min(Duration::from_millis(10))) {
            Ok(Command::OpenFloating(kind)) => floating.map(&connection, window.id, kind)?,
            Ok(Command::InspectFloating(kind)) => {
                observations.send(Ok(floating.inspect(&connection, kind)?))?;
            }
            Ok(Command::CloseFloating(kind)) => floating.close(&connection, kind)?,
            Ok(Command::Fullscreen) => window.fullscreen(&connection)?,
            Ok(Command::Inspect) => observations.send(Ok(window.inspect(&connection)?))?,
            Ok(Command::Stop) | Err(RecvTimeoutError::Disconnected) => {
                floating.close(&connection, FloatingKind::Splash)?;
                floating.close(&connection, FloatingKind::Transient)?;
                connection.destroy_window(window.id)?.check()?;
                connection.flush()?;
                return Ok(());
            }
            Err(RecvTimeoutError::Timeout) => (),
        }
    }
}
