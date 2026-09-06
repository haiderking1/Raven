use crate::state::State;
use smithay::{
    backend::renderer::utils::with_renderer_surface_state, desktop::Window,
    reexports::wayland_server::protocol::wl_surface::WlSurface, utils::IsAlive,
};

impl State {
    pub(crate) fn commit_window(&mut self, surface: &WlSurface) {
        let Some(window) = self
            .windows
            .iter()
            .find(|window| window.toplevel().is_some_and(|t| t.wl_surface() == surface))
            .cloned()
        else {
            return;
        };
        window.on_commit();
        let toplevel = window.toplevel().expect("Wayland window");
        let has_buffer =
            with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false);
        let mapped = self.space.element_location(&window).is_some();
        if mapped && !has_buffer {
            self.dismiss_window_popups(surface);
            window.set_activated(false);
            self.space.unmap_elem(&window);
            self.refresh_tiling();
            self.restore_focus();
            return;
        }
        if !toplevel.is_initial_configure_sent() {
            self.configure_initial_tile(&window);
            toplevel.send_configure();
        }
        if has_buffer && !mapped && toplevel.ensure_configured() {
            self.map_tiled_window(window.clone());
            // Do not interrupt a menu or drag that already owns keyboard focus.
            if !self.seat.get_keyboard().is_some_and(|k| k.is_grabbed()) {
                self.activate_window(Some(window));
            }
        }
    }

    pub(crate) fn remove_window(&mut self, window: &Window) {
        if let Some(toplevel) = window.toplevel() {
            self.dismiss_window_popups(toplevel.wl_surface());
        }
        self.space.unmap_elem(window);
        self.windows.retain(|candidate| candidate != window);
        self.refresh_tiling();
        self.restore_focus();
    }

    pub fn refresh(&mut self) {
        self.space.refresh();
        self.windows.retain(IsAlive::alive);
        self.refresh_tiling();
        self.popup_manager.cleanup();
        self.refresh_popup_grab();
        self.restore_focus();
    }
}
