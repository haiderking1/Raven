use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle},
};

#[derive(Clone, Debug)]
pub(crate) struct TileResize {
    windows: Vec<Window>,
    area: Rectangle<i32, Logical>,
    frames: Vec<Rectangle<i32, Logical>>,
    row: Option<(usize, bool)>,
}
impl TileResize {
    pub(crate) fn valid(&self, state: &State, index: usize) -> bool {
        state.tiling_area() == Some(self.area)
            && state.workspaces.entries[index].tiling.windows == self.windows
            && self
                .windows
                .iter()
                .all(|window| !state.fullscreen_manages(window))
    }
    pub(crate) fn contains(&self, window: &Window) -> bool {
        self.windows.contains(window)
    }
}
impl State {
    pub(crate) fn begin_tile_resize(&self, window: &Window, top: bool) -> Option<TileResize> {
        let index = self.workspaces.index_of(window)?;
        let tiling = &self.workspaces.entries[index].tiling;
        let count = tiling.windows.len();
        let area = self.tiling_area()?;
        if count == 0 || tiling.frames.len() != count {
            return None;
        }
        let slot = tiling
            .windows
            .iter()
            .position(|candidate| candidate == window)?;
        let row =
            (slot > 0 && count > 2).then(|| (slot - 1, (top && slot > 1) || slot == count - 1));
        let resize = TileResize {
            windows: tiling.windows.clone(),
            area,
            frames: tiling.frames.clone(),
            row,
        };
        resize.valid(self, index).then_some(resize)
    }

    pub(crate) fn resize_tiles(
        &mut self,
        index: usize,
        resize: &TileResize,
        delta: Point<i32, Logical>,
    ) {
        if !resize.valid(self, index)
            || resize.frames.len() < 2
            || resize.area.size.w < 2
            || resize.frames.len() - 1 > resize.area.size.h as usize
        {
            return;
        }
        let available = resize.frames[0].size.w + resize.frames[1].size.w;
        let master =
            ((resize.frames[0].size.w + delta.x) as f64 / available as f64).clamp(0.05, 0.95);
        let mut rows: Vec<f64> = resize.frames[1..]
            .iter()
            .map(|frame| frame.size.h as f64)
            .collect();
        if let Some((row, before)) = resize.row {
            super::geometry::rows::resize(&mut rows, row, before, delta.y);
        }
        let splits = &self.workspaces.entries[index].tiling.splits;
        if splits.master == master && splits.rows == rows {
            return;
        }
        self.begin_resize_batch(index);
        let splits = &mut self.workspaces.entries[index].tiling.splits;
        splits.master = master;
        splits.rows = rows;
        self.retile_workspace(index);
        self.end_resize_batch();
    }
}
