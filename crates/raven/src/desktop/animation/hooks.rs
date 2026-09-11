use crate::{backend::tty::TtyBackend, state::State};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{
        compositor::{
            BufferAssignment, SurfaceAttributes, add_destruction_hook, add_pre_commit_hook,
            with_states,
        },
        shell::xdg::{ToplevelConfigure, XdgToplevelSurfaceData},
    },
};

pub(crate) fn install(surface: &WlSurface) {
    add_pre_commit_hook::<State, _>(surface, |state, _, surface| pre_commit(state, surface));
    add_destruction_hook::<State, _>(surface, |state, surface| {
        state.cancel_surface_animation(surface)
    });
}

impl State {
    pub(crate) fn acknowledge_resize_animation(
        &mut self,
        surface: &WlSurface,
        configure: ToplevelConfigure,
    ) {
        if !self.animations.settings.enabled() {
            return;
        }
        let resize = with_states(surface, |states| {
            let Some(role) = states.data_map.get::<XdgToplevelSurfaceData>() else {
                return false;
            };
            let role = role.lock().unwrap();
            use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State as XdgState;
            role.current.size != configure.state.size
                || role.current.states.contains(XdgState::Fullscreen)
                    != configure.state.states.contains(XdgState::Fullscreen)
        });
        // Later ACKs supersede earlier ones, including activation-only configures
        // carrying the same not-yet-applied resize. Compare against current, not
        // the previous ACK, so those do not discard the resize intent.
        if resize {
            if let Some(context) = super::context::Context::of(self) {
                self.animations.acknowledged.insert(
                    surface.clone(),
                    super::context::Acknowledged {
                        serial: configure.serial,
                        context,
                    },
                );
            }
        } else {
            self.animations.acknowledged.remove(surface);
        }
    }

    pub(crate) fn cancel_surface_animation(&mut self, surface: &WlSurface) {
        self.animations.acknowledged.remove(surface);
        if let Some(backend) = self.backend.as_mut() {
            backend.cancel_resize_surface(surface);
        }
    }
}

fn pre_commit(state: &mut State, surface: &WlSurface) {
    let removed = with_states(surface, |states| {
        matches!(
            &states
                .cached_state
                .get::<SurfaceAttributes>()
                .pending()
                .buffer,
            Some(BufferAssignment::Removed)
        )
    });
    if removed {
        state.cancel_surface_animation(surface);
        return;
    }
    let Some(ack) = state.animations.acknowledged.remove(surface) else {
        return;
    };
    if super::context::Context::of(state).as_ref() != Some(&ack.context) {
        return;
    }
    if !state.animations.settings.enabled() || state.surface_on_hidden_workspace(surface) {
        return;
    }
    TtyBackend::capture_resize_animation(state, surface, ack.serial);
}
