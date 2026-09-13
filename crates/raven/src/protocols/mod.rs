mod compositor;
mod cursor_shape;
mod decorations;
pub(crate) use decorations::kde::KdeDecorations;
pub(crate) mod dmabuf;
mod layer_shell;
mod output;
mod pointer_constraints;
mod presentation;
mod relative_pointer;
mod seat;
mod selection;
mod shm;
mod viewport;
pub(crate) mod virtual_pointer;
#[path = "virtual_pointer/routing.rs"]
mod virtual_pointer_routing;
pub(crate) mod workspace;
mod xdg;
