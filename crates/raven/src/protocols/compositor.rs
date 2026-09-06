use crate::state::{ClientState, State};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor,
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
    fn commit(&mut self, surface: &WlSurface) {
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
        self.configure_popup(surface);
    }
    fn destroyed(&mut self, surface: &WlSurface) {
        use smithay::input::pointer::CursorImageStatus;
        if matches!(&self.cursor_status, CursorImageStatus::Surface(cursor) if cursor == surface) {
            self.cursor_status = CursorImageStatus::default_named();
        }
        if self.dnd_icon.as_ref() == Some(surface) {
            self.dnd_icon = None;
        }
    }
}
delegate_compositor!(State);
