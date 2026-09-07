mod arrange;
mod configure;
mod focus;
mod frames;
mod input;
mod lifecycle;
mod membership;

use smithay::{
    desktop::LayerSurface, output::Output,
    reexports::wayland_server::protocol::wl_surface::WlSurface, utils::Serial,
    wayland::shell::wlr_layer::Layer,
};
use std::collections::HashMap;

/// All accepted layers, including clients awaiting configure or a new buffer.
#[derive(Default)]
pub(crate) struct Layers {
    pub(crate) surfaces: Vec<LayerSurface>,
    entries: HashMap<WlSurface, Lifecycle>,
    changed: bool,
    dirty: bool,
    layout: Option<(
        Output,
        smithay::utils::Rectangle<i32, smithay::utils::Logical>,
    )>,
}

struct Lifecycle {
    output: Output,
    initial_layer: Layer,
    cycle: Option<ConfigureCycle>,
    /// Set by a root post-commit hook, before the renderer consumes its buffer assignment.
    commit: Option<bool>,
    mapped: bool,
}

struct ConfigureCycle {
    first: Serial,
    acknowledged: bool,
}

impl Layers {
    pub(crate) fn register(&mut self, surface: LayerSurface, output: Output, layer: Layer) {
        self.entries.insert(
            surface.wl_surface().clone(),
            Lifecycle {
                output,
                initial_layer: layer,
                cycle: None,
                commit: None,
                mapped: false,
            },
        );
        self.surfaces.push(surface);
        self.dirty = true;
    }

    pub(crate) fn note_commit(&mut self, surface: &WlSurface, null_buffer: bool) {
        if let Some(entry) = self.entries.get_mut(surface) {
            entry.commit = Some(null_buffer);
        }
    }

    /// Smithay retains old configures on unmap. Only this mapping cycle may be acknowledged.
    pub(crate) fn acknowledge(&mut self, surface: &WlSurface, serial: Serial) -> bool {
        let Some(cycle) = self
            .entries
            .get_mut(surface)
            .and_then(|entry| entry.cycle.as_mut())
        else {
            return false;
        };
        if serial < cycle.first {
            return false;
        }
        cycle.acknowledged = true;
        true
    }
}
