use crate::state::State;
use smithay::{delegate_output, wayland::output::OutputHandler};

// Space maintains surface enter/leave membership for the backend's output.
impl OutputHandler for State {
    fn output_bound(
        &mut self,
        _output: smithay::output::Output,
        _resource: smithay::reexports::wayland_server::protocol::wl_output::WlOutput,
    ) {
        // Called under output dispatch; defer object enumeration to reconciliation.
        self.workspace_protocol.outputs_changed();
    }
}
delegate_output!(State);
