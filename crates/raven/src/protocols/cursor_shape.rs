//! Named cursor requests use Smithay's seat focus and enter-serial validation.
use crate::state::State;
use smithay::{delegate_cursor_shape, wayland::tablet_manager::TabletSeatHandler};

// The delegate shares pointer/tablet dispatch bounds. Raven does not advertise
// a tablet manager; pointer requests use the existing SeatHandler::cursor_image.
impl TabletSeatHandler for State {}
delegate_cursor_shape!(State);
