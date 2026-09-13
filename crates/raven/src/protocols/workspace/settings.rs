use crate::{runtime::settings::WorkspaceSettings, state::State};
impl State {
    pub(crate) fn set_workspace_settings(&mut self, settings: &WorkspaceSettings) {
        self.workspace_protocol.show_all = settings.show_all;
        self.workspace_protocol.persistent.fill(false);
        for number in &settings.persistent {
            self.workspace_protocol.persistent[number - 1] = true;
        }
    }
}
