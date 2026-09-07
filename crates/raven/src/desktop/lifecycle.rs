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
        let Some(index) = self.workspaces.index_of(&window) else {
            return;
        };
        window.on_commit();
        let toplevel = window.toplevel().expect("Wayland window");
        let has_buffer =
            with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false);
        let mapped = self.workspaces.entries[index]
            .space
            .element_location(&window)
            .is_some();
        if mapped && !has_buffer {
            self.dismiss_window_popups(surface);
            window.set_activated(false);
            self.workspaces.entries[index].space.unmap_elem(&window);
            self.refresh_workspace_tiling(index);
            self.restore_focus();
            return;
        }
        if !toplevel.is_initial_configure_sent() {
            self.configure_initial_tile(&window);
            toplevel.send_configure();
        }
        if has_buffer && !mapped && toplevel.ensure_configured() {
            self.map_tiled_window(index, window.clone());
            // A delayed or remapped hidden client must never steal the seat.
            if index == self.workspaces.active
                && !self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
                && !self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
            {
                self.activate_window(Some(window));
            }
        }
    }

    pub(crate) fn remove_window(&mut self, window: &Window) {
        self.request_redraw();
        if let Some(toplevel) = window.toplevel() {
            self.dismiss_window_popups(toplevel.wl_surface());
        }
        if let Some(index) = self.workspaces.index_of(window) {
            self.workspaces.entries[index].space.unmap_elem(window);
            self.refresh_workspace_tiling(index);
        }
        self.workspaces.forget(window);
        self.windows.retain(|candidate| candidate != window);
        self.restore_focus();
    }

    pub fn refresh(&mut self) {
        self.workspaces.refresh();
        self.windows.retain(IsAlive::alive);
        self.refresh_layers();
        self.refresh_tiling();
        self.popup_manager.cleanup();
        self.refresh_popup_grab();
        self.restore_focus();
    }
}
