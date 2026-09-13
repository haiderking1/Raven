use crate::state::State;
impl State {
    pub(crate) fn set_bindings(&mut self, bindings: super::Bindings) {
        // Keep pressed-key disposition until release across reloads.
        self.input.shortcuts.bindings = bindings;
    }
    pub(crate) fn configuration_drag_active(&self) -> bool {
        self.input.drag.is_some()
    }
}
