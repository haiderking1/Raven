use crate::state::State;
use smithay::{
    input::pointer::ClickGrab,
    reexports::wayland_server::{Resource, Weak, protocol::wl_surface::WlSurface},
    utils::Serial,
    wayland::compositor::get_parent,
};

pub(super) struct Click {
    serial: Serial,
    focus: Weak<WlSurface>,
    pub(super) root: Weak<WlSurface>,
}

pub(super) enum Status {
    Held,
    Released,
    Cancelled,
}

impl Click {
    pub(super) fn held(state: &State) -> Option<Self> {
        if keyboard_grabbed(state) {
            return None;
        }
        // Extract only grab data under the pointer mutex. Layer/capture policy
        // must run after releasing it, since that policy can inspect the seat.
        let pointer = state.seat.get_pointer()?;
        let (serial, focus) = pointer.with_grab(|serial, grab| {
            if !grab.is::<ClickGrab<State>>() {
                return None;
            }
            grab.start_data()
                .focus
                .as_ref()
                .map(|(surface, _)| (serial, surface.clone()))
        })??;
        let root = root_of(focus.clone());
        if !focus.is_alive() || !mapped_layer(state, &root) {
            return None;
        }
        Some(Self {
            serial,
            focus: focus.downgrade(),
            root: root.downgrade(),
        })
    }

    pub(super) fn status(&self, state: &State) -> Status {
        let (Ok(focus), Ok(root)) = (self.focus.upgrade(), self.root.upgrade()) else {
            return Status::Cancelled;
        };
        if keyboard_grabbed(state) || root_of(focus) != root || !mapped_layer(state, &root) {
            return Status::Cancelled;
        }
        let Some(pointer) = state.seat.get_pointer() else {
            return Status::Cancelled;
        };
        pointer
            .with_grab(|serial, grab| {
                if serial == self.serial
                    && grab.is::<ClickGrab<State>>()
                    && grab
                        .start_data()
                        .focus
                        .as_ref()
                        .is_some_and(|(surface, _)| surface.id() == self.focus.id())
                {
                    Status::Held
                } else {
                    Status::Cancelled
                }
            })
            .unwrap_or(Status::Released)
    }
}

fn keyboard_grabbed(state: &State) -> bool {
    state
        .seat
        .get_keyboard()
        .is_some_and(|keyboard| keyboard.is_grabbed())
}

fn root_of(mut surface: WlSurface) -> WlSurface {
    while let Some(parent) = get_parent(&surface) {
        surface = parent;
    }
    surface
}

fn mapped_layer(state: &State, root: &WlSurface) -> bool {
    root.is_alive()
        && state
            .layers
            .surfaces
            .iter()
            .any(|layer| layer.wl_surface() == root && state.layer_is_mapped(layer))
}
