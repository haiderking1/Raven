//! Coordinated surface application and displayed allocations.
mod batch;
mod blocker;
mod callbacks;
mod commit;
mod configure;
mod fullscreen;
mod geometry;
mod lifecycle;
mod readiness;
mod release;
mod role;
mod runtime;

pub(crate) use role::apply as apply_role_state;

pub(crate) use commit::install_surface;
pub(crate) use readiness::acquire_started;

use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::calloop::{LoopHandle, RegistrationToken, ping::Ping},
    utils::{Logical, Point, Rectangle, Serial, Size},
};
use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Configure {
    serial: Serial,
    size: Option<Size<i32, Logical>>,
    fullscreen: bool,
}

pub(super) struct Held {
    frame: Rectangle<i32, Logical>,
    client: Rectangle<i32, Logical>,
    location: Point<i32, Logical>,
    target_location: Point<i32, Logical>,
    target_frame: Option<Rectangle<i32, Logical>>,
    target_client: Option<Rectangle<i32, Logical>>,
    configure: Option<Configure>,
    fullscreen_commits: Option<fullscreen::FullscreenCommits>,
    ready: Option<Vec<readiness::Readiness>>,
    applied: bool,
    queued: bool,
}

pub(super) struct Batch {
    workspace: usize,
    released: Arc<AtomicBool>,
    deadline: Instant,
    clients: Vec<smithay::reexports::wayland_server::Client>,
}

/// One visible cohort, one latest configure per mapped window, one deadline.
#[derive(Default)]
pub(crate) struct Transactions {
    held: HashMap<Window, Held>,
    callbacks: callbacks::Callbacks,
    fullscreen_notifications: Vec<smithay::reexports::wayland_server::Client>,
    batch: Option<Batch>,
    depth: usize,
    dirty: bool,
    collecting: bool,
    fresh: bool,
    releasing: bool,
    deferred_layout: [bool; crate::desktop::workspaces::COUNT],
    suspended: bool,
    output: Option<(smithay::output::Output, Rectangle<i32, Logical>)>,
    handle: Option<LoopHandle<'static, State>>,
    ping: Option<Ping>,
    timer: Option<RegistrationToken>,
}
