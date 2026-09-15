mod capabilities;
mod click;
mod fullscreen;
mod manager;
mod publish;
mod requests;
use smithay::{
    desktop::Window,
    reexports::{
        wayland_protocols_wlr::foreign_toplevel::v1::server::{
            zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1 as Handle,
            zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1 as Manager,
        },
        wayland_server::{
            Client, DisplayHandle, Resource, backend::GlobalId, protocol::wl_output::WlOutput,
        },
    },
};
use std::sync::atomic::{AtomicBool, Ordering};
pub(crate) struct ForeignToplevel {
    _global: GlobalId,
    subscriptions: Vec<Subscription>,
    restore_available: bool,
    pending_fullscreen: Option<Window>,
    pending_click: Option<click::Pending>,
}
struct Subscription {
    manager: Manager,
    client: Client,
    stopped: bool,
    entries: Vec<Entry>,
}
struct Entry {
    window: Window,
    handle: Handle,
    info: Option<Info>,
    outputs: Vec<WlOutput>,
}
#[derive(PartialEq)]
struct Info {
    title: String,
    app: String,
    states: Vec<u8>,
    parent: Option<Window>,
}
pub(crate) struct HandleData {
    window: Window,
    active: AtomicBool,
}
impl ForeignToplevel {
    pub(crate) fn new(display: &DisplayHandle) -> Self {
        Self {
            _global: display.create_global::<crate::state::State, Manager, _>(3, ()),
            subscriptions: Vec::new(),
            restore_available: false,
            pending_fullscreen: None,
            pending_click: None,
        }
    }
    pub(super) fn accepts_new_windows(&self) -> bool {
        self.subscriptions
            .iter()
            .any(|s| !s.stopped && s.manager.is_alive())
    }
    pub(crate) fn can_restore(&self, window: &Window) -> bool {
        self.has_managers()
            && self.subscriptions.iter().any(|s| {
                (!s.stopped
                    && s.manager.is_alive()
                    && !s.entries.iter().any(|e| e.window == *window))
                    || s.entries.iter().any(|e| {
                        e.window == *window
                            && e.handle.is_alive()
                            && e.handle
                                .data::<HandleData>()
                                .is_some_and(|d| d.active.load(Ordering::Relaxed))
                    })
            })
    }
    pub(crate) fn has_managers(&self) -> bool {
        self.subscriptions.iter().any(|s| {
            (!s.stopped && s.manager.is_alive())
                || s.entries.iter().any(|e| {
                    e.handle.is_alive()
                        && e.handle
                            .data::<HandleData>()
                            .is_some_and(|d| d.active.load(Ordering::Relaxed))
                })
        })
    }
}
