use crate::state::State;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities;
impl State {
    pub(super) fn refresh_management_capabilities(&mut self) {
        let available = self.foreign_toplevel.accepts_new_windows();
        let capabilities = |minimize| {
            let mut v = vec![WmCapabilities::Fullscreen, WmCapabilities::Maximize];
            if minimize {
                v.push(WmCapabilities::Minimize);
            }
            v
        };
        if available != self.foreign_toplevel.restore_available {
            self.foreign_toplevel.restore_available = available;
            self.xdg_shell_state
                .replace_capabilities(capabilities(available));
        }
        for top in self.xdg_shell_state.toplevel_surfaces() {
            let available = self
                .windows
                .iter()
                .find(|w| w.toplevel() == Some(top))
                .is_some_and(|w| self.foreign_toplevel.can_restore(w));
            let changed = top.with_pending_state(|state| {
                let old = state
                    .capabilities
                    .capabilities()
                    .any(|c| *c == WmCapabilities::Minimize);
                if old == available {
                    false
                } else {
                    state.capabilities.replace(capabilities(available));
                    true
                }
            });
            if changed && top.is_initial_configure_sent() {
                top.send_pending_configure();
            }
        }
    }
}
