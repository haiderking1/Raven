//! Process startup and the compositor event loop.
mod cli;
pub(crate) mod client;
mod environment;
mod flush;
mod signals;
mod wayland;

use crate::{backend::tty::TtyBackend, state::State};
use calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let Some(command) = cli::parse(std::env::args_os().skip(1))? else {
        return Ok(());
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .try_init()
        .map_err(|error| format!("cannot initialize logging: {error}"))?;
    environment::validate()?;
    let mut event_loop: EventLoop<'static, State> = EventLoop::try_new()?;
    // Block termination signals before EGL/libinput can create worker threads.
    signals::install(event_loop.handle())?;
    let display = Display::<State>::new()?;
    let mut state = State::new(display.handle(), event_loop.get_signal())?;
    let socket = wayland::install(display, event_loop.handle())?;
    // Capability detection and reservation finish before input/render installation.
    let mut clients = client::Clients::new(socket.clone());
    if let Err(error) = TtyBackend::install(&mut state, event_loop.handle()) {
        drop(state.backend.take());
        return Err(error);
    }
    eprintln!(
        "raven: listening on {} on {}; Super+Q opens foot, Super+D opens fuzzel, Super+C closes the focused window, Super+Shift+Q exits, Ctrl+Alt+Fn switches VT",
        socket.to_string_lossy(),
        state
            .backend
            .as_ref()
            .expect("backend installed")
            .seat_name()
    );
    eprintln!(
        "raven: Super+1..9/0 switches workspace; add Shift to move the focused window without following (0 selects workspace 10)"
    );
    if let Err(error) = clients.spawn(&command) {
        drop(state.backend.take());
        return Err(error);
    }
    state.clients = Some(clients);
    let mut flush_error = None;
    let result = event_loop.run(None, &mut state, |state| {
        // Deliver the input batch before desktop reconciliation or any GPU wait.
        if let Err(error) = flush::clients(state) {
            flush_error = Some(error);
            state.loop_signal.stop();
            return;
        }
        state.refresh();
        TtyBackend::dispatch(state);
        // Submission-time callbacks and presentation events must also go out now.
        if let Err(error) = flush::clients(state) {
            flush_error = Some(error);
            state.loop_signal.stop();
        }
    });
    let backend_error = state
        .backend
        .as_ref()
        .and_then(|backend| backend.failure().map(str::to_owned));
    // Restore KMS and release input while the libseat notifier is still alive.
    drop(state.backend.take());
    // Worker joins and child waits belong to shutdown, not the active compositor.
    drop(state.clients.take());
    result?;
    if let Some(error) = flush_error {
        return Err(error.into());
    }
    if let Some(error) = backend_error {
        return Err(error.into());
    }
    Ok(())
}
