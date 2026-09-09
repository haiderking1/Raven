//! Delegate viewport requests and double-buffered surface state to Smithay.
//! The compositor commit handler consumes that state before desktop updates.

use crate::state::State;
use smithay::delegate_viewporter;

delegate_viewporter!(State);
