use super::super::server::Server;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};
use std::error::Error;

pub(super) struct Baseline {
    main: Window,
    workspace: usize,
    tile: Rectangle<i32, Logical>,
    geometry: Rectangle<i32, Logical>,
    order: Vec<Window>,
    pub scene_size: usize,
}

impl Baseline {
    pub fn capture(server: &Server, main: &Window) -> Result<Self, Box<dyn Error>> {
        let tile = server
            .state
            .window_tile_geometry(main)
            .ok_or("main has no tile")?;
        let baseline = Self {
            main: main.clone(),
            workspace: server.state.workspaces.active,
            tile,
            geometry: main.geometry(),
            order: tile_order(server),
            scene_size: server.scene_size,
        };
        if baseline.order != [main.clone()]
            || baseline.scene_size != 1
            || server.state.workspace_floating_count(baseline.workspace) != 0
        {
            return Err("initial main window is not the sole tiled scene element".into());
        }
        baseline.check(server, 0)?;
        Ok(baseline)
    }

    pub fn check(&self, server: &Server, floaters: usize) -> Result<(), Box<dyn Error>> {
        let state = &server.state;
        let top = self.main.toplevel().ok_or("main toplevel disappeared")?;
        if state.workspaces.active != self.workspace
            || state.workspaces.index_of(&self.main) != Some(self.workspace)
            || state.window_is_floating(&self.main)
            || state.window_tile_geometry(&self.main) != Some(self.tile)
            || state.window_layout_geometry(&self.main) != Some(self.tile)
            || self.main.geometry() != self.geometry
            || self.geometry.size != self.tile.size
            || top.current_state().size != Some(self.tile.size)
            || state.space().element_location(&self.main) != Some(self.tile.loc)
            || tile_order(server) != self.order
            || state.workspace_floating_count(self.workspace) != floaters
            || state.fullscreen_window().is_some()
        {
            return Err(format!(
                "floating phase changed main tile size/order/count or floating count: tile={:?}, layout={:?}, geometry={:?}, configured={:?}, tiles={}, floaters={} expected={floaters}",
                state.window_tile_geometry(&self.main), state.window_layout_geometry(&self.main),
                self.main.geometry(), top.current_state().size, tile_order(server).len(),
                state.workspace_floating_count(self.workspace),
            ).into());
        }
        Ok(())
    }
}

fn tile_order(server: &Server) -> Vec<Window> {
    // The tiling storage is private. Its allocation accessor plus mapped order
    // proves this scenario retains exactly one full-sized tile, not a hidden
    // floater allocation or a second tile that happens to be obscured.
    server
        .state
        .visible_windows()
        .filter(|window| server.state.window_tile_geometry(window).is_some())
        .cloned()
        .collect()
}
