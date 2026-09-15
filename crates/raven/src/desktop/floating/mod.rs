mod access;
mod configure;
mod hints;
mod lifecycle;
pub(crate) mod maximize;
mod movement;
mod placement;
mod resizing;
mod stacking;
mod toggle;

use hints::Hints;
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle, Size},
};
use std::collections::{HashMap, HashSet};

/// Normal placement survives fullscreen; opening hints live only until mapping.
#[derive(Default)]
pub(crate) struct Floating {
    entries: HashMap<Window, Placement>,
    opening: HashMap<Window, Hints>,
    pub(crate) stack: Vec<Window>,
    pub(crate) elevated: HashSet<Window>,
}

#[derive(Clone, Default)]
struct Placement {
    manual_size: bool,
    position: Option<Point<i32, Logical>>,
    hints: Option<Hints>,
    natural: Option<Size<i32, Logical>>,
    geometry: Option<Rectangle<i32, Logical>>,
    frame: Option<Rectangle<i32, Logical>>,
}
