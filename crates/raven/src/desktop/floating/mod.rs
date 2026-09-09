mod access;
mod configure;
mod hints;
mod lifecycle;
mod placement;
mod stacking;

use hints::Hints;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle, Size},
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

#[derive(Default)]
struct Placement {
    hints: Option<Hints>,
    natural: Option<Size<i32, Logical>>,
    geometry: Option<Rectangle<i32, Logical>>,
}
