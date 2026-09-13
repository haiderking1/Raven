//! Process startup and the compositor event loop.
mod cli;
pub(crate) mod client;
pub(crate) mod config;
mod environment;
mod flush;
pub mod settings;
mod signals;
pub(crate) mod startup;
mod wayland;

use crate::{backend::tty::TtyBackend, state::State};
use calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    if let Some(result) = config::dialog::run_if_requested() {
        return result;
    }
    if let Some(result) = config::check_if_requested() {
        return result;
    }
    run_session(settings::Settings::default(), true)
}

/// Programmatic settings do not read or overwrite the user configuration file.
pub fn run_with_settings(settings: settings::Settings) -> Result<(), Box<dyn Error>> {
    run_session(settings, false)
}

fn run_session(settings: settings::Settings, use_file: bool) -> Result<(), Box<dyn Error>> {
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
    let defaults = settings.clone();
    let (prepared, config_path, initial_error) = config::initial(settings, use_file)?;
    let startup = prepared.settings.startup.clone().validate();
    environment::validate()?;
    let mut event_loop: EventLoop<'static, State> = EventLoop::try_new()?;
    // Block termination signals before EGL/libinput can create worker threads.
    signals::install(event_loop.handle())?;
    let display = Display::<State>::new()?;
    let mut state = State::new(display.handle(), event_loop.get_signal())?;
    state.install_resize_transactions(event_loop.handle())?;
    state.apply_configuration(prepared)?;
    let socket = wayland::install(display, event_loop.handle())?;
    // Capability detection and reservation finish before input/render installation.
    let mut clients = client::Clients::new(socket.clone());
    if let Err(error) = TtyBackend::install(&mut state, event_loop.handle()) {
        drop(state.backend.take());
        return Err(error);
    }
    eprintln!(
        "raven: listening on {} on {}",
        socket.to_string_lossy(),
        state
            .backend
            .as_ref()
            .expect("backend installed")
            .seat_name()
    );
    clients.start_startup(&startup);
    if let Err(error) = clients.spawn(&command) {
        drop(state.backend.take());
        return Err(error);
    }
    state.clients = Some(clients);
    let config_service = if let Some(path) = config_path {
        eprintln!("raven: configuration {}", path.display());
        match config::Service::install(
            path,
            defaults,
            &mut state,
            event_loop.handle(),
            socket.clone(),
        ) {
            Ok(service) => Some(service),
            Err(error) => {
                drop(state.backend.take());
                drop(state.clients.take());
                return Err(error);
            }
        }
    } else {
        None
    };
    if let Some(error) = initial_error {
        state.config.startup_error(error);
    }
    let mut flush_error = None;
    let result = event_loop.run(None, &mut state, |state| {
        // Deliver the input batch before desktop reconciliation or any GPU wait.
        if let Err(error) = flush::clients(state) {
            flush_error = Some(error);
            state.loop_signal.stop();
            return;
        }
        state.refresh();
        config::dispatch(state);
        state.refresh_workspace_protocol();
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
    drop(config_service);
    result?;
    if let Some(error) = flush_error {
        return Err(error.into());
    }
    if let Some(error) = backend_error {
        return Err(error.into());
    }
    Ok(())
}
