use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};

/// Displayed allocation, never a configure target or input coordinate system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub frame: Rectangle<i32, Logical>,
    pub client: Rectangle<i32, Logical>,
}

impl Geometry {
    pub fn of(state: &State, window: &Window) -> Option<Self> {
        let frame = state.window_frame_geometry(window)?;
        let client = state.window_client_geometry(window)?;
        if frame.is_empty() || client.is_empty() {
            return None;
        }
        Some(Self { frame, client })
    }

    pub fn interpolate(self, target: Self, progress: f64) -> Self {
        fn rect(
            a: Rectangle<i32, Logical>,
            b: Rectangle<i32, Logical>,
            p: f64,
        ) -> Rectangle<i32, Logical> {
            let mix =
                |a: i32, b: i32| (f64::from(a) + (f64::from(b) - f64::from(a)) * p).round() as i32;
            Rectangle::from_extremities(
                (mix(a.loc.x, b.loc.x), mix(a.loc.y, b.loc.y)),
                (
                    mix(a.loc.x + a.size.w, b.loc.x + b.size.w),
                    mix(a.loc.y + a.size.h, b.loc.y + b.size.h),
                ),
            )
        }
        Self {
            frame: rect(self.frame, target.frame, progress),
            client: rect(self.client, target.client, progress),
        }
    }
}
