use smithay::utils::{Logical, Rectangle};

use crate::state::State;

pub(super) fn bounds(state: &State) -> Option<Rectangle<i32, Logical>> {
    let output = state.output.as_ref()?;
    let bounds = state.space.output_geometry(output)?;
    (bounds.size.w > 0 && bounds.size.h > 0).then_some(bounds)
}
