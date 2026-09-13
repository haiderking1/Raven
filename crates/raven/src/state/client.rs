use smithay::{
    reexports::wayland_server::backend::ClientData, wayland::compositor::CompositorClientState,
};

/// Every Wayland client inserted into the display must carry this data.
#[derive(Debug, Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    pub(crate) configuration_error: bool,
}

impl ClientData for ClientState {}
