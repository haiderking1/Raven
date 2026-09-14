mod clipboard;
mod completion;
mod control;
pub(crate) mod encoding;
mod files;
mod input;
pub(crate) mod model;
mod native;
mod notice;
#[cfg(test)]
mod tests;
use crate::state::State;
use smithay::{
    backend::renderer::element::memory::MemoryRenderBuffer,
    reexports::calloop::{LoopHandle, RegistrationToken},
    utils::{Logical, Physical, Point, Rectangle, Size},
};
use std::collections::HashSet;
#[derive(Default)]
pub(crate) enum Phase {
    #[default]
    Idle,
    Requested,
    Selecting(model::Selection),
    Exporting(Rectangle<i32, Physical>),
}
#[derive(Default)]
pub(crate) struct Screenshot {
    pub phase: Phase,
    pub generation: u64,
    pub area: Rectangle<i32, Logical>,
    pub size: Size<i32, Physical>,
    pub scale: f64,
    pub commit: smithay::backend::renderer::utils::CommitCounter,
    origin: Point<f64, Logical>,
    pending_selection: Option<model::Selection>,
    confirm_pending: bool,
    buttons: HashSet<u32>,
    handle: Option<LoopHandle<'static, State>>,
    encoding: Option<encoding::Job>,
    pub notice: Option<MemoryRenderBuffer>,
    notice_timer: Option<RegistrationToken>,
    pub(crate) transfers: usize,
    transfer_bytes: usize,
    file_transfers: Vec<clipboard::FileTransfer>,
}
impl Screenshot {
    pub fn active(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }
}
