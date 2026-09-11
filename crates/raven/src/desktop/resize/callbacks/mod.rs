mod timer;

use crate::state::State;
use smithay::{
    reexports::{
        calloop::RegistrationToken,
        wayland_server::{
            Resource, Weak,
            protocol::{wl_callback::WlCallback, wl_surface::WlSurface},
        },
    },
    wayland::compositor::{SurfaceAttributes, get_children, is_sync_subsurface, with_states},
};
use std::time::Duration;

#[derive(Default)]
pub(super) struct Callbacks {
    pending: Vec<Pending>,
    timer: Option<RegistrationToken>,
}

struct Pending {
    surface: Weak<WlSurface>,
    owner: Weak<WlSurface>,
    callbacks: Vec<WlCallback>,
    queued: Duration,
}

impl State {
    pub(crate) fn resize_commit_queued(&mut self, surface: &WlSurface) {
        if self.resize.batch.is_none()
            || !self.resize.held.iter().any(|(window, held)| {
                held.configure.is_some()
                    && window
                        .toplevel()
                        .is_some_and(|top| top.wl_surface() == surface)
            })
        {
            return;
        }
        // This notification runs after Smithay snapshots the root and latches
        // synchronized descendants. Never inspect a child's pending requests or
        // revisit its cache on a timer: later child commits need another parent commit.
        let mut remaining = vec![surface.clone()];
        while let Some(node) = remaining.pop() {
            if &node != surface && !is_sync_subsurface(&node) {
                continue;
            }
            let callbacks = with_states(&node, |states| {
                let mut attributes = states.cached_state.get::<SurfaceAttributes>();
                let committed = attributes.take_committed_frame_callbacks();
                let mut callbacks = std::mem::take(&mut attributes.current().frame_callbacks);
                callbacks.extend(committed);
                if callbacks.is_empty() {
                    return callbacks;
                }
                drop(attributes);
                self.frame_callbacks.hold_coordination(states, true);
                callbacks
            });
            if !callbacks.is_empty() {
                let weak = node.downgrade();
                if let Some(pending) = self
                    .resize
                    .callbacks
                    .pending
                    .iter_mut()
                    .find(|p| p.surface == weak)
                {
                    pending.callbacks.extend(callbacks);
                } else {
                    self.resize.callbacks.pending.push(Pending {
                        surface: weak,
                        owner: surface.downgrade(),
                        callbacks,
                        queued: self.start_time.elapsed(),
                    });
                }
            }
            remaining.extend(get_children(&node));
        }
        self.arm_resize_callbacks();
    }
}
