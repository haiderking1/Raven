mod hooks;
mod lifecycle;
mod region;
mod restore;
mod surface;
#[cfg(test)]
mod tests;

pub(crate) use hooks::motion_hook;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

pub(crate) type Focus = Option<(WlSurface, Point<f64, Logical>)>;

#[derive(Clone, Debug)]
pub(crate) struct Active {
    pub surface: WlSurface,
    pub ancestors: Vec<WlSurface>,
    pub origin: Point<f64, Logical>,
    pub locked: bool,
    pub hint: Option<Point<f64, Logical>>,
}

/// Focus is recorded from Smithay's actual grab dispatch, never from a proposed hit target.
#[derive(Debug, Default)]
pub(crate) struct Capture {
    pub focus: Focus,
    pub active: Option<Active>,
    pub suspended: bool,
    pub keyboard_seen: bool,
    pub keyboard_focus: Option<WlSurface>,
}
