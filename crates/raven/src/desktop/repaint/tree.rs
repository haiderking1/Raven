use smithay::{
    backend::renderer::utils::{SurfaceView, with_renderer_surface_state},
    reexports::wayland_server::{Resource, Weak, protocol::wl_surface::WlSurface},
    utils::{Logical, Rectangle, Size, Transform},
    wayland::compositor::get_children,
};

#[derive(Default, PartialEq)]
pub(super) struct Geometry {
    pub(super) window: Option<Rectangle<i32, Logical>>,
    pub(super) nodes: Vec<Node>,
}

#[derive(PartialEq)]
pub(super) struct Node {
    surface: Weak<WlSurface>,
    parent: Option<Weak<WlSurface>>,
    view: Option<(
        Option<SurfaceView>,
        Option<Size<i32, Logical>>,
        i32,
        Transform,
    )>,
}

pub(super) fn snapshot(root: &WlSurface) -> (Vec<WlSurface>, Vec<Node>) {
    let mut pending = vec![(root.clone(), None)];
    let mut surfaces = Vec::new();
    let mut nodes = Vec::new();
    while let Some((surface, parent)) = pending.pop() {
        nodes.push(Node {
            surface: surface.downgrade(),
            parent,
            view: with_renderer_surface_state(&surface, |state| {
                (
                    state.view(),
                    state.buffer_size(),
                    state.buffer_scale(),
                    state.buffer_transform(),
                )
            }),
        });
        pending.extend(
            get_children(&surface)
                .into_iter()
                .filter(|child| child != &surface)
                .map(|child| (child, Some(surface.downgrade()))),
        );
        surfaces.push(surface);
    }
    (surfaces, nodes)
}
