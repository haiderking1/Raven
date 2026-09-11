pub mod animation;
pub mod appearance;
mod close;
pub(crate) mod floating;
mod focus;
pub(crate) mod frames;
pub(crate) mod fullscreen;
mod hit_test;
pub(crate) mod layers;
mod lifecycle;
mod pointer_focus;
mod popups;
mod presentation;
pub(crate) mod resize;
mod repaint;
pub(crate) mod tiling;
mod visibility;
pub(crate) mod workspaces;

#[cfg(test)]
pub(crate) mod tests;
