//! GPU snapshots live with Scene and are cancelled before its GLES device dies.
pub(super) mod accounting;
mod blend;
mod cache;
mod capture;
mod element;
mod frame;
mod lifecycle;
mod opacity;
mod paint;
mod progression;
mod resources;
mod snapshot;
mod tree;

pub(in crate::backend::tty::render) use blend::Blend;
pub(in crate::backend::tty) use cache::Animations;
pub(in crate::backend::tty::render) use paint::LiveElement;
