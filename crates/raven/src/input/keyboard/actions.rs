use crate::state::State;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    CycleApplications(bool),
    CancelAppSwitcher,
    Screenshot,
    ConfirmScreenshot,
    CancelScreenshot,
    ReloadConfig,
    Spawn(Vec<std::ffi::OsString>),
    CancelWindowDrag,
    SwitchVt(i32),
    LaunchTerminal,
    LaunchFuzzel,
    CloseWindow,
    ToggleFullscreen,
    ToggleFloating,
    SwitchWorkspace(usize),
    MoveToWorkspace(usize),
}

impl Action {
    pub(super) fn execute(self, state: &mut State) {
        match self {
            Self::Screenshot => {
                state.open_screenshot();
                return;
            }
            Self::ConfirmScreenshot => {
                state.confirm_screenshot();
                return;
            }
            Self::CancelScreenshot => {
                state.cancel_screenshot();
                return;
            }
            _ => {}
        }
        if let Self::CycleApplications(reverse) = self {
            state.cycle_applications(reverse);
            return;
        }
        state.cancel_screenshot();
        state.cancel_app_switcher();
        if self == Self::ReloadConfig {
            state.config.request_reload();
            return;
        }
        // A workspace/fullscreen/VT shortcut must not act through a move grab.
        state.cancel_window_drag();
        match self {
            Self::ReloadConfig
            | Self::CycleApplications(_)
            | Self::Screenshot
            | Self::ConfirmScreenshot
            | Self::CancelScreenshot => unreachable!(),
            Self::CancelAppSwitcher => {}
            Self::Spawn(argv) => launch(state, &argv),
            Self::CancelWindowDrag => state.cancel_window_drag(),
            Self::CloseWindow => state.close_focused_window(),
            Self::ToggleFullscreen => state.toggle_fullscreen(),
            Self::ToggleFloating => state.toggle_focused_floating(),
            Self::SwitchWorkspace(index) => state.switch_workspace(index),
            Self::MoveToWorkspace(index) => state.move_focused_to_workspace(index),
            Self::Quit => {
                state.loop_signal.stop();
                state.loop_signal.wakeup();
            }
            Self::SwitchVt(vt) => {
                if let Some(backend) = state.backend.as_mut()
                    && let Err(error) = backend.change_vt(vt)
                {
                    eprintln!("Failed to switch to VT {vt}: {error}");
                }
            }
            Self::LaunchTerminal | Self::LaunchFuzzel => {
                let argv = if self == Self::LaunchTerminal {
                    state.config.settings.terminal.clone()
                } else {
                    state.config.settings.launcher.clone()
                };
                launch(state, &argv);
            }
        }
    }
}

fn launch(state: &mut State, argv: &[std::ffi::OsString]) {
    if let Some(clients) = state.clients.as_mut() {
        if let Err(error) = clients.spawn(argv) {
            eprintln!("raven: {error}");
        }
    }
}
