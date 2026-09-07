use crate::state::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Quit,
    SwitchVt(i32),
    LaunchTerminal,
    LaunchFuzzel,
    CloseWindow,
    SwitchWorkspace(usize),
    MoveToWorkspace(usize),
}

impl Action {
    pub(super) fn execute(self, state: &mut State) {
        match self {
            Self::CloseWindow => state.close_focused_window(),
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
                let program = if self == Self::LaunchTerminal {
                    "foot"
                } else {
                    "fuzzel"
                };
                let Some(clients) = state.clients.as_mut() else {
                    eprintln!("raven: cannot launch {program} before the Wayland socket is ready");
                    return;
                };
                if let Err(error) = clients.spawn(&[program.into()]) {
                    eprintln!("raven: {error}");
                }
            }
        }
    }
}
