//! Presentation timestamps use CLOCK_MONOTONIC, including the fallback path.
pub(crate) mod commit;
use crate::state::State;
smithay::delegate_presentation!(State);
