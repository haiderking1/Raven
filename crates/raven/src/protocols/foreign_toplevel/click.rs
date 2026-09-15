use super::{Handle, HandleData};
use crate::state::State;
use smithay::{
    desktop::Window,
    input::pointer::ClickGrab,
    reexports::wayland_server::{Resource, Weak},
};
use std::sync::atomic::Ordering;
pub(super) struct Pending {
    window: Window,
    fullscreen: bool,
    handle: Weak<Handle>,
}
impl State {
    pub(crate) fn cancel_management_requests(&mut self) {
        self.foreign_toplevel.pending_click = None;
        self.foreign_toplevel.pending_fullscreen = None;
    }
    pub(super) fn cancel_management_request_for(&mut self, window: &Window) {
        if self
            .foreign_toplevel
            .pending_click
            .as_ref()
            .is_some_and(|p| p.window == *window)
        {
            self.foreign_toplevel.pending_click = None;
        }
        if self.foreign_toplevel.pending_fullscreen.as_ref() == Some(window) {
            self.foreign_toplevel.pending_fullscreen = None;
        }
    }
    pub(super) fn request_taskbar_action(
        &mut self,
        window: &Window,
        fullscreen: bool,
        handle: &Handle,
    ) {
        self.cancel_management_requests();
        self.foreign_toplevel.pending_click = Some(Pending {
            window: window.clone(),
            fullscreen,
            handle: handle.downgrade(),
        });
        self.refresh_taskbar_click();
    }
    pub(super) fn refresh_taskbar_click(&mut self) {
        if self.foreign_toplevel.pending_click.is_none() {
            return;
        }
        if self.screenshot.active()
            || self.exclusive_keyboard_layer().is_some()
            || self.pointer_is_captured()
            || self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
        {
            self.foreign_toplevel.pending_click = None;
            return;
        }
        let alive = self
            .foreign_toplevel
            .pending_click
            .as_ref()
            .and_then(|p| p.handle.upgrade().ok())
            .is_some_and(|h| {
                h.is_alive()
                    && h.data::<HandleData>()
                        .is_some_and(|d| d.active.load(Ordering::Relaxed))
            });
        if !alive {
            self.foreign_toplevel.pending_click = None;
            return;
        }
        if let Some(pointer) = self.seat.get_pointer() {
            if pointer.is_grabbed() {
                let click = pointer
                    .with_grab(|_, grab| grab.is::<ClickGrab<State>>())
                    .unwrap_or(false);
                if !click {
                    self.foreign_toplevel.pending_click = None;
                }
                return;
            }
        }
        let pending = self.foreign_toplevel.pending_click.take().unwrap();
        if pending.fullscreen {
            self.request_managed_fullscreen(&pending.window);
        } else {
            self.activate_managed_window(&pending.window);
        }
    }
}
