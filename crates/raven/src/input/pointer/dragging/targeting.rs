use super::Drag;
use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{IsAlive, Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

impl Drag {
    pub(super) fn valid(&self, state: &State) -> bool {
        self.window.alive()
            && state.workspaces.active == self.workspace
            && state.workspaces.index_of(&self.window) == Some(self.workspace)
            && state.window_is_visible(&self.window)
            && state.space().element_location(&self.window).is_some()
            && !state.fullscreen_manages(&self.window)
            && state.window_is_floating(&self.window) == self.floating
            && (self.floating || state.window_is_tiled(&self.window))
            && state.output.as_ref() == Some(&self.output)
            && state.space().output_geometry(&self.output) == Some(self.area)
            && self.output.current_scale().fractional_scale() == self.scale
            && self.output.current_transform() == self.transform
            && state
                .window_frame_geometry(&self.window)
                .is_some_and(|frame| {
                    if let Some(resize) = self.resize.as_ref() {
                        resize.valid(state, self.workspace)
                    } else if self.floating {
                        frame.size == self.frame.size
                    } else {
                        frame == self.frame
                    }
                })
    }

    pub(super) fn target(&self, state: &State, point: Point<f64, Logical>) -> Option<Window> {
        if self.resize.is_some()
            || self.floating
            || !self.valid(state)
            || state
                .layer_under(point, &[Layer::Overlay, Layer::Top])
                .is_some()
        {
            return None;
        }
        let window = state.window_focus_under(point)?;
        (window != &self.window
            && state.window_is_tiled(window)
            && !state.fullscreen_manages(window))
        .then(|| window.clone())
    }
}

impl State {
    pub(crate) fn surface_is_dragged_window(
        &self,
        root: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> bool {
        self.input.drag.as_ref().is_some_and(|drag| {
            self.windows.iter().any(|window| {
                window
                    .toplevel()
                    .is_some_and(|top| top.wl_surface() == root)
                    && (window == &drag.window
                        || drag
                            .resize
                            .as_ref()
                            .and_then(|resize| resize.tiles.as_ref())
                            .is_some_and(|tiles| tiles.contains(window)))
            })
        })
    }

    pub(crate) fn dragged_tile(&self) -> Option<(&Window, Point<i32, Logical>)> {
        let drag = self.input.drag.as_ref()?;
        (drag.resize.is_none() && !drag.floating && drag.valid(self)).then(|| {
            (
                &drag.window,
                (self.pointer_location - drag.start).to_i32_round(),
            )
        })
    }
}
