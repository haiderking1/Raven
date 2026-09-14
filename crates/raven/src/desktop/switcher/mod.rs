mod activation;
pub(crate) mod artwork;
mod catalogue;
pub(crate) mod layout;
mod lifecycle;
pub(crate) mod model;
mod repeat;
#[cfg(test)]
mod tests;

use smithay::{desktop::Window, input::keyboard::Keycode};
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct Switcher {
    history: Vec<Window>,
    activating: bool,
    session: Option<model::Session<Window>>,
    pub(crate) pending: Option<Window>,
    exiting: Option<Window>,
    pub(crate) artwork: artwork::Artwork,
    pub(crate) swallowed_buttons: HashSet<u32>,
    pub(crate) repeat_key: Option<Keycode>,
    reverse: bool,
    repeat_follows_shift: bool,
    repeat: repeat::Repeat,
}
impl Switcher {
    pub(crate) fn active(&self) -> bool {
        self.session.is_some()
    }
}
