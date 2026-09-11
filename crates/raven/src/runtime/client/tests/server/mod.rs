mod pointer;
mod renderer;
pub(super) mod sockets;

use crate::{runtime::wayland, state::State};
use calloop::EventLoop;
use smithay::{
    backend::renderer::gles::GlesRenderer,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_server::Display,
    utils::Rectangle,
};
use std::{
    error::Error,
    path::PathBuf,
    time::{Duration, Instant},
};

pub(super) struct Server {
    pub state: State,
    pub socket: PathBuf,
    pub area: Rectangle<i32, smithay::utils::Logical>,
    pub scene_size: usize,
    event_loop: EventLoop<'static, State>,
    renderer: GlesRenderer,
}

impl Server {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let runtime = PathBuf::from(
            std::env::var_os("XDG_RUNTIME_DIR").ok_or("XDG_RUNTIME_DIR is required")?,
        );
        if !runtime.is_absolute() {
            return Err("XDG_RUNTIME_DIR must be absolute".into());
        }
        let event_loop = EventLoop::try_new()?;
        let display = Display::<State>::new()?;
        let mut state = State::new(display.handle(), event_loop.get_signal())?;
        let output = Output::new(
            "satellite-integration".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "Raven test".into(),
                model: "Headless".into(),
            },
        );
        let mode = Mode {
            size: (960, 540).into(),
            refresh: 60_000,
        };
        output.change_current_state(Some(mode), None, None, Some((0, 0).into()));
        output.set_preferred(mode);
        output.create_global::<State>(&state.display_handle);
        state.space_mut().map_output(&output, (0, 0));
        state.output = Some(output);
        let area = state
            .fullscreen_area()
            .ok_or("virtual output has no geometry")?;
        let socket = runtime.join(wayland::install(display, event_loop.handle())?);
        let renderer = renderer::create()?;
        Ok(Self {
            state,
            socket,
            area,
            scene_size: 0,
            event_loop,
            renderer,
        })
    }

    pub fn until(
        &mut self,
        deadline: Instant,
        condition: &str,
        mut ready: impl FnMut(&Self) -> Result<bool, Box<dyn Error>>,
    ) -> Result<(), Box<dyn Error>> {
        loop {
            self.state.display_handle.flush_clients()?;
            let remaining = deadline.checked_duration_since(Instant::now())
                .ok_or_else(|| format!("deadline waiting for {condition}; windows={}, mapped={}, scene={}, fullscreen={}",
                    self.state.windows.len(), self.state.space().elements().count(),
                    self.scene_size, self.state.fullscreen_window().is_some()))?;
            self.event_loop.dispatch(
                Some(remaining.min(Duration::from_millis(10))),
                &mut self.state,
            )?;
            self.state.display_handle.flush_clients()?;
            self.state.refresh();
            self.state.refresh_workspace_protocol();
            // Exercise production desktop import. Only this real scene build earns callbacks.
            self.scene_size = crate::backend::tty::test_scene_size(&mut self.renderer, &self.state);
            self.state.send_frames(self.state.start_time.elapsed());
            self.state.display_handle.flush_clients()?;
            if ready(self)? {
                return Ok(());
            }
        }
    }
}
