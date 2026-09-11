//! Resize visuals are independent of commit blockers and displayed allocations.
mod context;
pub(crate) mod geometry;
mod hooks;
pub(crate) mod participation;
mod settings;
pub(crate) mod timeline;

pub use settings::{InvalidAnimationDuration, ResizeAnimations};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Animations {
    pub settings: ResizeAnimations,
    pub participation: std::cell::RefCell<Vec<participation::GroupParticipation>>,
    /// One latest ACK per root. ACKs themselves neither capture nor move content.
    acknowledged: HashMap<WlSurface, context::Acknowledged>,
}

pub(crate) use hooks::install;

impl Animations {
    pub fn clear_intents(&mut self) {
        self.acknowledged.clear();
        self.participation.borrow_mut().clear();
    }
}

impl crate::state::State {
    pub fn resize_animations(&self) -> ResizeAnimations {
        self.animations.settings
    }

    pub(crate) fn cancel_resize_visuals(&mut self) {
        self.animations.clear_intents();
        if let Some(backend) = self.backend.as_mut() {
            backend.cancel_resize_animations();
        }
    }

    pub fn set_resize_animations(&mut self, settings: ResizeAnimations) {
        if self.animations.settings == settings {
            return;
        }
        self.animations.settings = settings;
        self.animations.clear_intents();
        if let Some(backend) = self.backend.as_mut() {
            backend.cancel_resize_animations();
        }
        self.request_redraw();
    }
}
