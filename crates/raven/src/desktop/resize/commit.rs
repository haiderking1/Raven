use super::{Configure, blocker::Coordination, readiness, role};
use crate::state::State;
use smithay::{
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    },
    wayland::{
        compositor::{
            BufferAssignment, SurfaceAttributes, add_blocker, add_destruction_hook,
            add_pre_commit_hook, with_states,
        },
        shell::xdg::XdgToplevelSurfaceData,
    },
};

pub(crate) fn install_surface(surface: &WlSurface) {
    add_pre_commit_hook::<State, _>(surface, pre_commit);
    add_destruction_hook::<State, _>(surface, |state, surface| {
        state.cancel_resize_surface(surface)
    });
}

fn pre_commit(state: &mut State, _: &DisplayHandle, surface: &WlSurface) {
    role::capture(surface);
    let unmapping = with_states(surface, |states| {
        matches!(
            states
                .cached_state
                .get::<SurfaceAttributes>()
                .pending()
                .buffer,
            Some(BufferAssignment::Removed)
        )
    });
    if unmapping {
        state.cancel_resize_surface(surface);
        return;
    }
    let Some(window) = state
        .resize
        .held
        .keys()
        .find(|w| w.toplevel().is_some_and(|top| top.wl_surface() == surface))
        .cloned()
    else {
        return;
    };
    // Position-only siblings retain layout but need not stall ordinary content.
    if state.resize.held[&window].configure.is_none() {
        return;
    }
    let acknowledged = with_states(surface, |states| {
        let data = states.data_map.get::<XdgToplevelSurfaceData>()?;
        let role = data.lock().unwrap();
        let expected = state.resize.held[&window].configure?;
        let serial = role.configure_serial?;
        let acked = role.last_acked.as_ref()?;
        let fullscreen = acked.states.contains(xdg_toplevel::State::Fullscreen);
        let desired = role
            .server_pending
            .as_ref()
            .unwrap_or_else(|| role.current_server_state());
        // Activation/decorations can flush server_pending outside layout code.
        // Accept their newer serial only if it carries our issued or newest
        // target, never merely because its integer serial is newer.
        let issued = acked.size == expected.size && fullscreen == expected.fullscreen;
        let latest = acked.size == desired.size
            && fullscreen == desired.states.contains(xdg_toplevel::State::Fullscreen);
        (serial >= expected.serial && (issued || latest)).then_some(Configure {
            serial,
            size: acked.size,
            fullscreen,
        })
    });
    if let Some(acknowledged) = acknowledged {
        state.resize.held.get_mut(&window).unwrap().configure = Some(acknowledged);
        state.resize.held.get_mut(&window).unwrap().ready = Some(readiness::tree(surface));
        state.send_queued_resize(&window);
    }
    if let Some(batch) = &state.resize.batch {
        // Older replies are gated too. They must not replace current content
        // while the displayed allocation still describes the old layout.
        add_blocker(surface, Coordination(batch.released.clone()));
        state.wake_resize_transactions();
    }
}

impl State {
    /// Called after commit_window, only when Smithay actually applied state.
    pub(crate) fn resize_surface_applied(&mut self, surface: &WlSurface) {
        let Some(window) = self
            .resize
            .held
            .keys()
            .find(|w| w.toplevel().is_some_and(|top| top.wl_surface() == surface))
            .cloned()
        else {
            return;
        };
        let applied = with_states(surface, |states| {
            let Some(data) = states.data_map.get::<XdgToplevelSurfaceData>() else {
                return false;
            };
            let role = data.lock().unwrap();
            self.resize.held[&window].configure.is_none_or(|expected| {
                role.current_serial
                    .is_some_and(|serial| serial >= expected.serial)
                    && role.current.size == expected.size
                    && role
                        .current
                        .states
                        .contains(xdg_toplevel::State::Fullscreen)
                        == expected.fullscreen
            })
        });
        if applied {
            self.resize.held.get_mut(&window).unwrap().applied = true;
            if !self.resize.releasing && self.resize.batch.is_none() {
                self.publish_resize_windows(vec![window]);
            }
        }
    }
}
