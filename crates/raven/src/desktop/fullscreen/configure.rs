use super::{Target, Transition};
use crate::desktop::appearance::configure_client_decorations;
use crate::state::State;
use smithay::{desktop::Window, reexports::wayland_protocols::xdg::shell::server::xdg_toplevel};

impl State {
    pub(crate) fn configure_fullscreen(&mut self, window: &Window, force: bool) {
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let Some(toplevel) = window.toplevel() else {
            return;
        };
        if !toplevel.is_initial_configure_sent() {
            return;
        }
        let Some(entry) = self.workspaces.entries[index]
            .fullscreen
            .entries
            .get(window)
        else {
            if force {
                toplevel.send_configure();
            }
            return;
        };
        let full = entry.intent && self.fullscreen_area().is_some();
        let target = Target {
            fullscreen: full,
            geometry: if full {
                self.fullscreen_area()
            } else {
                self.floating_geometry(window)
                    .or_else(|| self.window_tile_geometry(window))
            },
        };
        if entry
            .transition
            .as_ref()
            .is_some_and(|t| t.target == target)
        {
            // A client request still needs a configure response, but an unchanged
            // target must not move the serial we are waiting for. The client may
            // already be committing the original configure.
            if force {
                set_pending(window, target, self.window_is_floating(window));
                toplevel.send_configure();
            }
            return;
        }
        if !force && entry.transition.is_none() && entry.applied == target.geometry.filter(|_| full)
        {
            return;
        }
        self.begin_resize_batch(index);
        set_pending(window, target, self.window_is_floating(window));
        let serial = toplevel.send_configure();
        self.track_fullscreen_resize_configure(window, serial);
        self.workspaces.entries[index]
            .fullscreen
            .entries
            .get_mut(window)
            .unwrap()
            .transition = Some(Transition {
            serial,
            target,
            committed: false,
        });
        self.capture_fullscreen_animation(window, serial);
        self.end_resize_batch();
    }

    /// Called only for the first bufferless commit of each mapping cycle.
    pub(crate) fn configure_initial_fullscreen(&mut self, window: &Window) {
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let toplevel = window.toplevel().expect("Wayland window");
        let full = self.workspaces.entries[index]
            .fullscreen
            .entries
            .get(window)
            .is_some_and(|entry| entry.intent)
            && self.fullscreen_area().is_some();
        if full {
            let target = Target {
                fullscreen: true,
                geometry: self.fullscreen_area(),
            };
            set_pending(window, target, self.window_is_floating(window));
            let serial = toplevel.send_configure();
            self.workspaces.entries[index]
                .fullscreen
                .entries
                .get_mut(window)
                .unwrap()
                .transition = Some(Transition {
                serial,
                target,
                committed: false,
            });
        } else {
            toplevel.send_configure();
        }
    }
}

fn set_pending(window: &Window, target: Target, floating: bool) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    toplevel.with_pending_state(|state| {
        state.size = target.geometry.map(|area| area.size);
        state.bounds = target.geometry.map(|area| area.size);
        // Raven has one output. A client resource hint is not retained across
        // output removal or resource destruction; placement uses the live output.
        state.fullscreen_output = None;
        if target.fullscreen {
            state.states.set(xdg_toplevel::State::Fullscreen);
        } else {
            state.states.unset(xdg_toplevel::State::Fullscreen);
        }
        for edge in [
            xdg_toplevel::State::TiledLeft,
            xdg_toplevel::State::TiledRight,
            xdg_toplevel::State::TiledTop,
            xdg_toplevel::State::TiledBottom,
        ] {
            // Fullscreen changes the size and mode, not the underlying layout
            // or decoration policy. Toggling tiled edges here makes GTK clients
            // reconfigure CSD and shadow geometry during the fullscreen resize.
            if floating {
                state.states.unset(edge);
            } else {
                state.states.set(edge);
            }
        }
    });
    configure_client_decorations(window);
}
