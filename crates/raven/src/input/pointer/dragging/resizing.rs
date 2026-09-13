use super::Drag;
use crate::{desktop::tiling::TileResize, state::State};
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{IsAlive, Logical, Point, Rectangle},
};

#[derive(Debug)]
pub(super) struct Resize {
    pub left: bool,
    pub top: bool,
    pub tiles: Option<TileResize>,
    workarea: Rectangle<i32, Logical>,
    last: Point<f64, Logical>,
    pending: Option<Point<f64, Logical>>,
}
impl Resize {
    pub(super) fn new(
        state: &State,
        window: &Window,
        frame: Rectangle<i32, Logical>,
        floating: bool,
    ) -> Option<Self> {
        let point = state.pointer_location;
        let left = point.x < frame.loc.x as f64 + frame.size.w as f64 / 2.0;
        let top = point.y < frame.loc.y as f64 + frame.size.h as f64 / 2.0;
        let tiles = if floating {
            None
        } else {
            Some(state.begin_tile_resize(window, top)?)
        };
        Some(Self {
            left,
            top,
            tiles,
            workarea: state.tiling_area()?,
            last: point,
            pending: None,
        })
    }
    pub(super) fn valid(&self, state: &State, index: usize) -> bool {
        state.tiling_area() == Some(self.workarea)
            && self
                .tiles
                .as_ref()
                .is_none_or(|tiles| tiles.valid(state, index))
    }
    pub(super) fn motion(&mut self, point: Point<f64, Logical>) {
        if point != self.last {
            self.last = point;
            self.pending = Some(point);
        }
    }
}

pub(super) fn set_resizing(state: &mut State, window: &Window, active: bool) {
    if !active {
        state.reset_floating_resize_pacing();
    }
    if !window.alive() {
        return;
    }
    if let Some(top) = window.toplevel() {
        top.with_pending_state(|pending| {
            if active {
                pending.states.set(xdg_toplevel::State::Resizing);
            } else {
                pending.states.unset(xdg_toplevel::State::Resizing);
            }
        });
        state.send_resize_configure(window);
    }
}
impl Drag {
    /// Grab callbacks cannot send layout configures: these can refresh pointer focus.
    pub(super) fn queue_resize_cleanup(&self, state: &mut State) {
        if self.resize.is_some() {
            state.input.resize_cleanup = Some(self.window.clone());
        }
    }
    pub(super) fn finish_resize(&self, state: &mut State) {
        if self.resize.is_some() {
            set_resizing(state, &self.window, false);
        }
    }
}
impl State {
    /// Run only after returning from Smithay's pointer mutex.
    pub(crate) fn apply_window_resize(&mut self) {
        let floating = self
            .input
            .drag
            .as_ref()
            .filter(|drag| {
                drag.floating
                    && drag
                        .resize
                        .as_ref()
                        .is_some_and(|resize| resize.pending.is_some())
            })
            .map(|drag| drag.window.clone());
        if floating.is_some_and(|window| self.defer_floating_resize(&window)) {
            return;
        }
        let Some(drag) = self.input.drag.as_mut() else {
            return;
        };
        let Some(resize) = drag.resize.as_mut() else {
            return;
        };
        let Some(point) = resize.pending.take() else {
            return;
        };
        let window = drag.window.clone();
        let frame = drag.frame;
        let index = drag.workspace;
        let delta = (point - drag.start).to_i32_round();
        let (left, top, tiles) = (resize.left, resize.top, resize.tiles.clone());
        if let Some(tiles) = tiles {
            self.resize_tiles(index, &tiles, delta);
        } else {
            self.resize_floating_window(&window, frame, delta, left, top);
        }
    }
}
