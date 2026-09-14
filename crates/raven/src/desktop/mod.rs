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
mod repaint;
pub(crate) mod resize;
pub(crate) mod screenshot;
pub(crate) mod switcher;
pub(crate) mod tiling;
mod visibility;
pub(crate) mod workspaces;

#[cfg(test)]
pub(crate) mod tests;
