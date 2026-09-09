use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Size},
    wayland::{compositor::with_states, shell::xdg::SurfaceCachedState},
};

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Hints {
    pub min: Size<i32, Logical>,
    pub max: Size<i32, Logical>,
    parent: Option<WlSurface>,
}

impl Hints {
    pub(super) fn committed(window: &Window) -> Self {
        let top = window.toplevel().expect("Wayland window");
        let (min, max) = with_states(top.wl_surface(), |states| {
            let mut cached = states.cached_state.get::<SurfaceCachedState>();
            let current = cached.current();
            (current.min_size, current.max_size)
        });
        Self {
            min,
            max,
            parent: top.parent(),
        }
    }

    fn floats(&self) -> bool {
        self.parent.is_some() || (self.min.h > 0 && self.min.h == self.max.h)
    }
}

impl State {
    /// Only called before mapping: ordinary mapped commits never change mode.
    pub(crate) fn prepare_floating(&mut self, window: &Window) -> bool {
        let Some(mut index) = self.workspaces.index_of(window) else {
            return false;
        };
        if let Some(parent) = self.valid_floating_parent(window)
            && let Some(destination) = self.workspaces.index_of(parent)
            && destination != index
        {
            self.transfer_unmapped_fullscreen(window, index, destination);
            self.transfer_floating(window, index, destination);
            self.workspaces.assign(window.clone(), destination);
            index = destination;
        }
        let hints = Hints::committed(window);
        let floating = &mut self.workspaces.entries[index].floating;
        if floating.opening.get(window) == Some(&hints) {
            return false;
        }
        if hints.floats() {
            floating.entries.entry(window.clone()).or_default();
        } else {
            floating.entries.remove(window);
        }
        floating.opening.insert(window.clone(), hints);
        true
    }
}
