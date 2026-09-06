use crate::state::{ClientState, State};
use calloop::{Interest, LoopHandle, Mode, PostAction, generic::Generic};
use smithay::{reexports::wayland_server::Display, wayland::socket::ListeningSocketSource};
use std::{error::Error, ffi::OsString, sync::Arc};

pub(super) fn install(
    display: Display<State>,
    handle: LoopHandle<'static, State>,
) -> Result<OsString, Box<dyn Error>> {
    let socket = ListeningSocketSource::new_auto()?;
    let name = socket.socket_name().to_owned();
    handle.insert_source(socket, |stream, _, state| {
        if let Err(error) = state
            .display_handle
            .insert_client(stream, Arc::new(ClientState::default()))
        {
            eprintln!("raven: could not accept Wayland client: {error}");
        }
    })?;
    handle.insert_source(
        Generic::new(display, Interest::READ, Mode::Level),
        |_, display, state| {
            // SAFETY: dispatch mutates the display without dropping/replacing it or
            // closing its poll fd, which remains registered for the source lifetime.
            unsafe { display.get_mut() }.dispatch_clients(state)?;
            Ok(PostAction::Continue)
        },
    )?;
    Ok(name)
}
