use super::hints::Hints;
use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Size},
};

/// Zero means client choice only before the first natural size is known.
pub(super) fn bounded_size(
    hints: &Hints,
    natural: Option<Size<i32, Logical>>,
    bounds: Option<Size<i32, Logical>>,
) -> Size<i32, Logical> {
    fn axis(natural: Option<i32>, min: i32, max: i32, bound: Option<i32>) -> i32 {
        let requested = natural
            .filter(|v| *v > 0)
            .unwrap_or_else(|| if min > 0 && min == max { min } else { 0 });
        if requested == 0 {
            return 0;
        }
        let lower = min.max(1);
        let upper = if max > 0 { max.max(lower) } else { i32::MAX };
        // Workarea wins when a client's minimum cannot fit on the output.
        requested
            .clamp(lower, upper)
            .min(bound.unwrap_or(i32::MAX).max(1))
    }
    (
        axis(
            natural.map(|s| s.w),
            hints.min.w,
            hints.max.w,
            bounds.map(|s| s.w),
        ),
        axis(
            natural.map(|s| s.h),
            hints.min.h,
            hints.max.h,
            bounds.map(|s| s.h),
        ),
    )
        .into()
}

impl State {
    pub(crate) fn configure_floating(&self, window: &Window) -> bool {
        let Some(index) = self.workspaces.index_of(window) else {
            return false;
        };
        let Some(entry) = self.workspaces.entries[index].floating.entries.get(window) else {
            return false;
        };
        let bounds = self.tiling_area().map(|area| area.size);
        let size = bounded_size(&Hints::committed(window), entry.natural, bounds);
        let top = window.toplevel().expect("Wayland window");
        top.with_pending_state(|state| {
            state.size = Some(size);
            state.bounds = bounds;
            state.fullscreen_output = None;
            for flag in [
                xdg_toplevel::State::Fullscreen,
                xdg_toplevel::State::Maximized,
                xdg_toplevel::State::TiledLeft,
                xdg_toplevel::State::TiledRight,
                xdg_toplevel::State::TiledTop,
                xdg_toplevel::State::TiledBottom,
            ] {
                state.states.unset(flag);
            }
        });
        true
    }
}
