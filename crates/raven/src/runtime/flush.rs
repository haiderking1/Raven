use crate::state::State;
use std::io;

/// A flush is an attempt to write queued protocol events, not proof of client receipt.
pub(super) fn clients(state: &mut State) -> io::Result<()> {
    let boundary = state
        .input_timing
        .as_mut()
        .map(|timing| timing.before_flush());
    let result = state.display_handle.flush_clients();
    if let (Some(timing), Some(boundary)) = (&mut state.input_timing, boundary) {
        timing.after_flush(boundary, result.is_ok());
    }
    result
}
