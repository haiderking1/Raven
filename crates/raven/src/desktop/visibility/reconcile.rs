use crate::state::State;

impl State {
    /// A missing or changed ancestor can hide an entire mapped dialog chain.
    pub(crate) fn reconcile_transient_visibility(&mut self) {
        let hidden: Vec<_> = self
            .space()
            .elements()
            .filter(|window| !self.window_is_visible(window))
            .filter_map(|window| window.toplevel().map(|t| t.wl_surface().clone()))
            .collect();
        for root in hidden {
            self.dismiss_window_popups(&root);
        }
        self.refresh_popup_grab();
        self.restore_focus();
        self.request_redraw();
        self.refresh_tiling_pointer();
    }
}
