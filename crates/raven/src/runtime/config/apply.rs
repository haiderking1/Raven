use super::{Prepared, Runtime};
use crate::state::State;

impl State {
    pub(crate) fn apply_configuration(&mut self, prepared: Prepared) -> Result<(), String> {
        // Candidate settings and cursor assets were validated before publication.
        let Prepared { settings, cursor } = prepared;
        self.set_input_settings(settings.input)?;
        self.set_appearance(settings.appearance)
            .expect("validated appearance");
        self.set_resize_animations(settings.resize_animations);
        self.set_bindings(settings.bindings.clone());
        self.set_workspace_settings(&settings.workspaces);
        if let Some(backend) = &mut self.backend {
            if self.config.settings.cursor != settings.cursor {
                backend.set_cursor_settings(cursor);
            }
        } else {
            self.config.initial_cursor = Some(cursor);
        }
        self.config.settings = settings;
        self.config.dialog.close();
        self.request_redraw();
        Ok(())
    }
}

pub(crate) fn dispatch(state: &mut State) {
    state.config.dialog.reap();
    let Some(pending) = state.config.pending.take() else {
        return;
    };
    match pending {
        Ok(prepared) => {
            if !state.configuration_layout_ready() {
                state.config.pending = Some(Ok(prepared));
                return;
            }
            match state.apply_configuration(prepared) {
                Ok(()) => eprintln!("raven: configuration reloaded"),
                Err(error) => {
                    state.config.pending = Some(Err(error));
                    dispatch(state);
                }
            }
        }
        Err(error) => {
            eprintln!("raven: configuration rejected: {error}");
            let report = format!(
                "Last working settings remain active.

{error}"
            );
            if let Some(socket) = state.config.socket.clone()
                && let Err(error) =
                    state
                        .config
                        .dialog
                        .show(report, &socket, &mut state.display_handle)
            {
                eprintln!("raven: cannot show configuration error window: {error}");
            }
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            pending: None,
            initial_cursor: None,
            socket: None,
            dialog: Default::default(),
            control: None,
        }
    }
}
