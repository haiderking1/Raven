mod client;
mod init;

pub use client::ClientState;

use smithay::{
    desktop::{PopupGrab, PopupManager, Window},
    input::{Seat, SeatState, pointer::CursorImageStatus},
    output::Output,
    reexports::{
        calloop::LoopSignal,
        wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    },
    utils::{Logical, Point},
    wayland::{
        compositor::CompositorState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{XdgShellState, decoration::XdgDecorationState},
        },
        shm::ShmState,
    },
};
use std::time::Instant;

pub struct State {
    pub display_handle: DisplayHandle,
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
    pub xdg_shell_state: XdgShellState,
    pub layer_shell_state: WlrLayerShellState,
    pub(crate) layers: crate::desktop::layers::Layers,
    // Retain the global handle; XDG decoration dispatch requires no state getter.
    pub(crate) _xdg_decoration_state: XdgDecorationState,
    pub data_device_state: DataDeviceState,
    pub seat_state: SeatState<Self>,
    pub seat: Seat<Self>,
    pub output: Option<Output>,
    pub start_time: Instant,
    pub loop_signal: LoopSignal,
    pub pointer_location: Point<f64, Logical>,
    pub cursor_status: CursorImageStatus,
    pub backend: Option<crate::backend::tty::TtyBackend>,
    pub input: crate::input::InputState,
    pub(crate) clients: Option<crate::runtime::client::Clients>,
    pub(crate) workspaces: crate::desktop::workspaces::Workspaces,
    pub popup_manager: PopupManager,
    pub(crate) popup_grab: Option<(WlSurface, PopupGrab<Self>)>,
    /// Includes unmapped toplevels so a null-buffer commit can later remap them.
    pub(crate) windows: Vec<Window>,
    /// The renderer draws this surface at the pointer during a client drag.
    pub dnd_icon: Option<WlSurface>,
}
