use crate::state::State;
use smithay::{
    output::Output,
    utils::{Logical, Rectangle, Transform},
};

/// An ACK must not carry a resize visual across an output or workspace change.
#[derive(Clone, PartialEq)]
pub(super) struct Context {
    output: Output,
    area: Rectangle<i32, Logical>,
    scale: f64,
    transform: Transform,
    workspace: usize,
}

impl Context {
    pub fn of(state: &State) -> Option<Self> {
        let output = state.output.as_ref()?;
        Some(Self {
            output: output.clone(),
            area: state.space().output_geometry(output)?,
            scale: output.current_scale().fractional_scale(),
            transform: output.current_transform(),
            workspace: state.workspaces.active,
        })
    }
}

pub(super) struct Acknowledged {
    pub serial: smithay::utils::Serial,
    pub context: Context,
}
