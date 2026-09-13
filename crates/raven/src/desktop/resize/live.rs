use crate::{backend::tty::TtyBackend, state::State};
use smithay::{desktop::Window, output::Output, utils::IsAlive};

pub(super) struct Live {
    workspace: usize,
    floating: bool,
    output: Output,
}
impl State {
    pub(crate) fn begin_live_resize(&mut self, window: &Window) {
        let Some(workspace) = self.workspaces.index_of(window) else {
            return;
        };
        let Some(output) = self.output.clone() else {
            return;
        };
        if self.fullscreen_manages(window) {
            return;
        }
        TtyBackend::prepare_live_resize(window);
        self.cancel_resize_window(window);
        self.resize.live.insert(
            window.clone(),
            Live {
                workspace,
                floating: self.window_is_floating(window),
                output,
            },
        );
        if let Some(client) = self.window_target_client_geometry(window) {
            self.workspaces.entries[workspace]
                .space
                .map_element(window.clone(), client.loc, false);
        }
        // Cancellation can leave the latest server-pending size unsent.
        self.send_resize_configure(window);
    }

    pub(crate) fn end_live_resize(&mut self, window: &Window) {
        self.resize.live.remove(window);
    }

    pub(crate) fn window_has_live_resize(&self, window: &Window) -> bool {
        self.resize.live.get(window).is_some_and(|live| {
            window.alive()
                && live.workspace == self.workspaces.active
                && self.workspaces.index_of(window) == Some(live.workspace)
                && self.space().element_location(window).is_some()
                && self.output.as_ref() == Some(&live.output)
                && self.window_is_floating(window) == live.floating
                && !self.fullscreen_manages(window)
        })
    }

    pub(super) fn refresh_live_resizes(&mut self) {
        let remove: Vec<_> = self
            .resize
            .live
            .keys()
            .filter(|window| {
                if !self.window_has_live_resize(window) {
                    return true;
                }
                let active = window
                    .toplevel()
                    .is_some_and(|top| self.surface_is_dragged_window(top.wl_surface()));
                !active
                    && self
                        .window_target_client_geometry(window)
                        .is_some_and(|target| {
                            window.geometry().size == target.size
                                && window.toplevel().is_some_and(|top| {
                                    top.current_state().size == Some(target.size)
                                })
                                && TtyBackend::live_resize_content_settled(window, target.size)
                        })
            })
            .cloned()
            .collect();
        for window in remove {
            self.resize.live.remove(&window);
        }
    }
}
