//! Smithay 0.7's XDG post-hook reads the newest ACK at application time.
//! Cache the ACK with its wl_surface commit so queued commits cannot borrow a
//! newer mode or serial. No vendor change and no independent commit queue.
use smithay::utils::Serial;
use smithay::{
    reexports::wayland_server::{DisplayHandle, protocol::wl_surface::WlSurface},
    wayland::{
        compositor::{Cacheable, with_states},
        shell::xdg::{ToplevelState, XdgToplevelSurfaceData},
    },
};

#[derive(Clone, Default)]
struct RoleCommit {
    acknowledged: Option<(Serial, ToplevelState)>,
}

impl Cacheable for RoleCommit {
    fn commit(&mut self, _: &DisplayHandle) -> Self {
        self.clone()
    }
    fn merge_into(self, into: &mut Self, _: &DisplayHandle) {
        *into = self;
    }
}

pub(super) fn capture(surface: &WlSurface) {
    with_states(surface, |states| {
        let Some(data) = states.data_map.get::<XdgToplevelSurfaceData>() else {
            return;
        };
        let role = data.lock().unwrap();
        let acknowledged = role.configure_serial.zip(role.last_acked.clone());
        states
            .cached_state
            .get::<RoleCommit>()
            .pending()
            .acknowledged = acknowledged;
    });
}

/// First operation in CompositorHandler::commit, after Smithay's role hook.
/// Existing fullscreen/animation consumers then see the commit's actual ACK.
pub(crate) fn apply(surface: &WlSurface) {
    with_states(surface, |states| {
        let Some(data) = states.data_map.get::<XdgToplevelSurfaceData>() else {
            return;
        };
        let acknowledged = states
            .cached_state
            .get::<RoleCommit>()
            .current()
            .acknowledged
            .clone();
        if let Some((serial, current)) = acknowledged {
            let mut role = data.lock().unwrap();
            role.current_serial = Some(serial);
            role.current = current;
        }
    });
}
