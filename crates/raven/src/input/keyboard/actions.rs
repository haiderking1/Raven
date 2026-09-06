use crate::state::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Quit,
    SwitchVt(i32),
    LaunchTerminal,
    CloseWindow,
}

impl Action {
    pub(super) fn execute(self, state: &mut State) {
        match self {
            Self::CloseWindow => state.close_focused_window(),
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
            Self::LaunchTerminal => {
                let Some(clients) = state.clients.as_mut() else {
                    eprintln!("raven: cannot launch foot before the Wayland socket is ready");
                    return;
                };
                if let Err(error) = clients.spawn(&["foot".into()]) {
                    eprintln!("raven: {error}");
                }
            }
        }
    }
}
