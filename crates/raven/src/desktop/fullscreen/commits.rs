use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

impl State {
    pub(crate) fn commit_fullscreen(&mut self, window: &Window) {
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let Some(toplevel) = window.toplevel() else {
            return;
        };
        let serial = with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|data| data.lock().unwrap().current_serial)
        });
        let current = toplevel.current_state();
        if let Some(transition) = self.workspaces.entries[index]
            .fullscreen
            .entries
            .get_mut(window)
            .and_then(|entry| entry.transition.as_mut())
            && serial.is_some_and(|serial| serial >= transition.serial)
            && current.states.contains(xdg_toplevel::State::Fullscreen)
                == transition.target.fullscreen
            && current.size == transition.target.geometry.map(|area| area.size)
        {
            // Serial comparison is Smithay's wrap-aware comparison. An activation
            // configure carrying the same mode may be newer than our transition.
            transition.committed = true;
        }
        self.apply_fullscreen_transitions(index);
        self.position_fullscreen_windows(index);
    }
}
