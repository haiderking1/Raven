mod delegation;
mod subsurface;
use crate::state::{ClientState, State};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    reexports::wayland_server::{Client, protocol::wl_surface::WlSurface},
    wayland::compositor::{
        CompositorClientState, CompositorHandler, CompositorState, get_parent, is_sync_subsurface,
    },
};

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("Wayland client missing raven::state::ClientState")
            .compositor_state
    }
    fn new_surface(&mut self, surface: &WlSurface) {
        crate::protocols::dmabuf::acquire::install(surface);
        crate::desktop::resize::install_surface(surface);
        crate::protocols::presentation::commit::install(surface);
        crate::desktop::animation::install(surface);
    }
    fn commit_queued(&mut self, surface: &WlSurface) {
        self.resize_commit_queued(surface);
    }
    fn commit(&mut self, surface: &WlSurface) {
        crate::desktop::resize::apply_role_state(surface);
        on_commit_buffer_handler::<Self>(surface);
        self.popup_manager.commit(surface);
        if is_sync_subsurface(surface) {
            return;
        }
        let mut root = surface.clone();
        while let Some(parent) = get_parent(&root) {
            root = parent;
        }
        self.commit_window(&root);
        self.resize_surface_applied(&root);
        self.commit_layer(&root);
        self.configure_popup(surface);
        if !self.surface_on_hidden_workspace(&root) {
            self.request_redraw();
        }
    }
    fn destroyed(&mut self, surface: &WlSurface) {
        self.request_redraw();
        self.remove_layer(surface);
        use smithay::input::pointer::CursorImageStatus;
        if matches!(&self.cursor_status, CursorImageStatus::Surface(cursor) if cursor == surface) {
            self.cursor_status = CursorImageStatus::default_named();
        }
        if self.dnd_icon.as_ref() == Some(surface) {
            self.dnd_icon = None;
        }
    }
}
