use super::State;
use smithay::{
    desktop::PopupManager,
    input::{SeatState, pointer::CursorImageStatus},
    reexports::{calloop::LoopSignal, wayland_server::DisplayHandle},
    wayland::{
        compositor::CompositorState,
        dmabuf::DmabufState,
        presentation::PresentationState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{XdgShellState, decoration::XdgDecorationState},
        },
        shm::ShmState,
    },
};
use std::{error::Error, time::Instant};

impl State {
    pub fn new(
        display_handle: DisplayHandle,
        loop_signal: LoopSignal,
    ) -> Result<Self, Box<dyn Error>> {
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let presentation_state =
            PresentationState::new::<Self>(&display_handle, libc::CLOCK_MONOTONIC as u32);
        // Do not advertise window-management operations we do not implement.
        let xdg_shell_state = XdgShellState::new_with_capabilities::<Self>(&display_handle, vec![]);
        let layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&display_handle, "seat0");
        seat.add_keyboard(Default::default(), 400, 25)?;
        seat.add_pointer();
        Ok(Self {
            display_handle,
            compositor_state,
            shm_state,
            _presentation_state: presentation_state,
            dmabuf_state: DmabufState::new(),
            xdg_shell_state,
            layer_shell_state,
            layers: Default::default(),
            _xdg_decoration_state: xdg_decoration_state,
            data_device_state,
            seat_state,
            seat,
            output: None,
            start_time: Instant::now(),
            frame_callbacks: Default::default(),
            loop_signal,
            pointer_location: (0.0, 0.0).into(),
            cursor_status: CursorImageStatus::default_named(),
            backend: None,
            redraw_requested: false,
            input: Default::default(),
            input_timing: crate::input::timing::InputTiming::from_env(),
            clients: None,
            workspaces: Default::default(),
            popup_manager: PopupManager::default(),
            popup_grab: None,
            windows: Vec::new(),
            dnd_icon: None,
        })
    }
}
