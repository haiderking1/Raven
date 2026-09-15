use super::{Entry, Handle, HandleData, Info};
use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::wayland_server::Resource,
    utils::IsAlive,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};
use std::sync::atomic::{AtomicBool, Ordering};
impl State {
    pub(crate) fn refresh_foreign_toplevels(&mut self) {
        if self.foreign_toplevel.subscriptions.is_empty()
            && self.foreign_toplevel.pending_click.is_none()
            && self.foreign_toplevel.pending_fullscreen.is_none()
            && !self.foreign_toplevel.restore_available
        {
            return;
        }
        self.refresh_taskbar_click();
        self.refresh_managed_fullscreen();
        let windows: Vec<_> = self
            .windows
            .iter()
            .filter(|w| w.alive() && !self.is_configuration_error(w))
            .filter(|w| {
                self.workspaces.index_of(w).is_some_and(|i| {
                    self.workspaces.entries[i]
                        .space
                        .element_location(w)
                        .is_some()
                })
            })
            .cloned()
            .collect();
        let mut subscriptions = std::mem::take(&mut self.foreign_toplevel.subscriptions);
        for subscription in &mut subscriptions {
            subscription.entries.retain(|entry| {
                if windows.contains(&entry.window) {
                    return true;
                }
                if entry.handle.is_alive() {
                    entry
                        .handle
                        .data::<HandleData>()
                        .unwrap()
                        .active
                        .store(false, Ordering::Relaxed);
                    entry.handle.closed();
                }
                false
            });
            if !subscription.stopped && subscription.manager.is_alive() {
                for window in &windows {
                    if subscription.entries.iter().any(|e| e.window == *window) {
                        continue;
                    }
                    let Ok(handle) = subscription.client.create_resource::<Handle, _, Self>(
                        &self.display_handle,
                        subscription.manager.version(),
                        HandleData {
                            window: window.clone(),
                            active: AtomicBool::new(true),
                        },
                    ) else {
                        continue;
                    };
                    subscription.manager.toplevel(&handle);
                    subscription.entries.push(Entry {
                        window: window.clone(),
                        handle,
                        info: None,
                        outputs: Vec::new(),
                    });
                }
            }
            let parents: Vec<_> = subscription
                .entries
                .iter()
                .filter(|e| e.handle.is_alive())
                .map(|e| (e.window.clone(), e.handle.clone()))
                .collect();
            for entry in &mut subscription.entries {
                if !entry.handle.is_alive() {
                    continue;
                }
                let mut info = self.foreign_info(&entry.window, entry.handle.version());
                info.parent = info
                    .parent
                    .filter(|w| parents.iter().any(|(parent, _)| parent == w));
                let mut changed = false;
                if entry
                    .info
                    .as_ref()
                    .is_none_or(|old| old.title != info.title)
                {
                    entry.handle.title(info.title.clone());
                    changed = true;
                }
                if entry.info.as_ref().is_none_or(|old| old.app != info.app) {
                    entry.handle.app_id(info.app.clone());
                    changed = true;
                }
                if entry
                    .info
                    .as_ref()
                    .is_none_or(|old| old.states != info.states)
                {
                    entry.handle.state(info.states.clone());
                    changed = true;
                }
                if entry.handle.version() >= 3
                    && entry
                        .info
                        .as_ref()
                        .is_none_or(|old| old.parent != info.parent)
                {
                    entry.handle.parent(
                        info.parent
                            .as_ref()
                            .and_then(|w| parents.iter().find(|(p, _)| p == w).map(|(_, h)| h)),
                    );
                    changed = true;
                }
                let outputs: Vec<_> = self
                    .output
                    .as_ref()
                    .map(|o| {
                        o.client_outputs(&subscription.client)
                            .filter(Resource::is_alive)
                            .collect()
                    })
                    .unwrap_or_default();
                for output in &entry.outputs {
                    if output.is_alive() && !outputs.contains(output) {
                        entry.handle.output_leave(output);
                        changed = true;
                    }
                }
                for output in &outputs {
                    if !entry.outputs.contains(output) {
                        entry.handle.output_enter(output);
                        changed = true;
                    }
                }
                entry.outputs = outputs;
                entry.info = Some(info);
                if changed {
                    entry.handle.done();
                }
            }
        }
        subscriptions.retain(|s| {
            (!s.stopped && s.manager.is_alive()) || s.entries.iter().any(|e| e.handle.is_alive())
        });
        self.foreign_toplevel.subscriptions = subscriptions;
        self.refresh_management_capabilities();
    }
    fn foreign_info(&self, window: &Window, version: u32) -> Info {
        let (app, title) = window
            .toplevel()
            .and_then(|top| {
                with_states(top.wl_surface(), |states| {
                    let data = states
                        .data_map
                        .get::<XdgToplevelSurfaceData>()?
                        .lock()
                        .ok()?;
                    Some((
                        data.app_id.clone().unwrap_or_default(),
                        data.title.clone().unwrap_or_default(),
                    ))
                })
            })
            .unwrap_or_default();
        let mut states = Vec::new();
        for (enabled, value) in [
            (
                crate::desktop::floating::maximize::is_maximized(window),
                0u32,
            ),
            (self.window_is_minimized(window), 1),
            (self.focused_window().as_ref() == Some(window), 2),
            (
                version >= 2 && self.applied_fullscreen_allocation(window).is_some(),
                3,
            ),
        ] {
            if enabled {
                states.extend(value.to_ne_bytes());
            }
        }
        Info {
            app: app.chars().take(4096).collect(),
            title: title.chars().take(4096).collect(),
            states,
            parent: self.valid_floating_parent(window).cloned(),
        }
    }
}
