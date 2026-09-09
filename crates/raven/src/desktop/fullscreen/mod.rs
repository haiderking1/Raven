mod apply;
mod commits;
mod configure;
mod geometry;
mod grabs;
mod placement;
mod refresh;
mod requests;
mod transfer;
mod transients;

use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle, Serial},
};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Fullscreen {
    pub(crate) requested: Option<Window>,
    pub(crate) displayed: Option<Window>,
    entries: HashMap<Window, Entry>,
}

#[derive(Default)]
struct Entry {
    // Unmapped requests are intent only. They claim the workspace on mapping.
    intent: bool,
    applied: Option<Rectangle<i32, Logical>>,
    transition: Option<Transition>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Target {
    fullscreen: bool,
    geometry: Option<Rectangle<i32, Logical>>,
}

struct Transition {
    serial: Serial,
    target: Target,
    // Set only in Raven's commit handler, after Smithay's post-commit hook.
    committed: bool,
}
