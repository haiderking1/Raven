//! Presentation timestamps use CLOCK_MONOTONIC, including the fallback path.
use crate::state::State;
smithay::delegate_presentation!(State);
