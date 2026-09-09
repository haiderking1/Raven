//! Optional policy hook around the actual motion selected by a pointer grab.
use super::{MotionEvent, PointerHandle};
use crate::{
    input::{Seat, SeatHandler},
    utils::{Logical, Point},
};

/// Whether a motion is about to be delivered or has been delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionPhase {
    /// Return false to suppress this motion, without changing pointer state.
    Before,
    /// The focus and location now describe the delivered motion. Return value is ignored.
    After,
}

/// Runs under the pointer mutex. Do not call methods that lock the pointer from this hook.
/// Both targets are the actual grab-selected targets, not the pending hit-test target.
pub type MotionHook<D> = fn(
    &mut D,
    &Seat<D>,
    MotionPhase,
    &Option<(<D as SeatHandler>::PointerFocus, Point<f64, Logical>)>,
    &Option<(<D as SeatHandler>::PointerFocus, Point<f64, Logical>)>,
    &MotionEvent,
) -> bool;

impl<D: SeatHandler + 'static> PointerHandle<D> {
    /// Install a motion policy hook. The hook must not re-lock this pointer.
    pub fn set_motion_hook(&self, hook: MotionHook<D>) {
        self.inner.lock().unwrap().motion_hook = Some(hook);
    }
}
