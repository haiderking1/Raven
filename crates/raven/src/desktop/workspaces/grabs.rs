use crate::state::State;

impl State {
    pub(super) fn prepare_workspace_change(&mut self) -> bool {
        // Dismiss a menu before hiding its toplevel and release its seat grabs.
        if let Some((root, _)) = &self.popup_grab {
            let root = root.clone();
            self.dismiss_window_popups(&root);
        }
        // Keep an in-progress selection or DnD on its workspace until release.
        // Otherwise the destination could receive a release without a press.
        !self
            .seat
            .get_keyboard()
            .is_some_and(|keyboard| keyboard.is_grabbed())
            && !self
                .seat
                .get_pointer()
                .is_some_and(|pointer| pointer.is_grabbed())
    }
}
