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
        let Some(mut index) = self.workspaces.index_of(&window) else {
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
            self.cancel_resize_window(&window);
            self.dismiss_window_popups(surface);
            window.set_activated(false);
            self.clear_fullscreen(&window);
            self.workspaces.entries[index].space.unmap_elem(&window);
            self.forget_floating(&window);
            self.request_redraw();
            self.refresh_tiling_pointer();
            self.refresh_workspace_tiling(index);
            self.retile_workspace(index);
            self.reconcile_transient_visibility();
            return;
        }
        let opening_changed = if !mapped {
            let changed = self.prepare_floating(&window);
            index = self.workspaces.index_of(&window).expect("assigned window");
            changed
        } else {
            false
        };
        if !toplevel.is_initial_configure_sent() {
            self.configure_initial_tile(&window);
            self.configure_initial_fullscreen(&window);
        } else if opening_changed {
            if self.fullscreen_manages(&window) {
                self.configure_fullscreen(&window, true);
            } else {
                self.configure_initial_tile(&window);
                toplevel.send_pending_configure();
            }
        }
        if has_buffer && !mapped && toplevel.ensure_configured() {
            self.map_layout_window(index, window.clone());
            self.map_fullscreen_intent(index, &window);
            self.commit_fullscreen(&window);
            // A delayed or remapped hidden client must never steal the seat.
            if index == self.workspaces.active
                && self.window_is_visible(&window)
                && !self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
                && !self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
            {
                self.activate_window(Some(window.clone()));
            }
        } else if has_buffer && mapped {
            let floating_changed = self.commit_floating_size(&window);
            self.commit_fullscreen(&window);
            if floating_changed {
                self.arrange_floating(index);
            }
        }
    }

    pub(crate) fn remove_window(&mut self, window: &Window) {
        self.cancel_resize_window(window);
        self.request_redraw();
        self.clear_fullscreen(window);
        if let Some(toplevel) = window.toplevel() {
            self.dismiss_window_popups(toplevel.wl_surface());
        }
        if let Some(index) = self.workspaces.index_of(window) {
            self.workspaces.entries[index].space.unmap_elem(window);
            self.refresh_workspace_tiling(index);
            self.retile_workspace(index);
        }
        self.forget_floating(window);
        self.workspaces.forget(window);
        self.windows.retain(|candidate| candidate != window);
        // Orphans keep floating mode but recenter without a stale parent anchor.
        for index in 0..self.workspaces.entries.len() {
            self.arrange_floating(index);
        }
        self.reconcile_transient_visibility();
    }

    pub(crate) fn subsurface_role_removed(&mut self) {
        // Smithay keeps the removed role's surface private. Refresh live window
        // tree bounds on this rare lifecycle event, never on every input event.
        for window in self.windows.iter().filter(|window| window.alive()) {
            window.on_commit();
        }
        let windows = self.windows.clone();
        for window in windows {
            self.commit_floating_size(&window);
        }
        for index in 0..self.workspaces.entries.len() {
            self.position_fullscreen_windows(index);
            self.arrange_floating(index);
        }
        self.request_redraw();
        self.refresh_tiling_pointer();
    }

    pub fn refresh(&mut self) {
        self.refresh_resize_lifecycle();
        self.workspaces.refresh();
        self.windows.retain(IsAlive::alive);
        self.refresh_layers();
        self.refresh_tiling();
        self.refresh_fullscreen();
        self.refresh_floating();
        self.popup_manager.cleanup();
        self.refresh_popup_grab();
        self.restore_focus();
    }
}
