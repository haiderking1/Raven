use crate::state::State;
use smithay::{delegate_output, wayland::output::OutputHandler};

// Space maintains surface enter/leave membership for the backend's output.
impl OutputHandler for State {}
delegate_output!(State);
