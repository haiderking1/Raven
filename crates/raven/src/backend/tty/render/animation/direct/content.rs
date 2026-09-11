use smithay::{
    backend::renderer::{
        element::{Element, Id, surface::WaylandSurfaceRenderElement},
        gles::GlesRenderer,
        utils::with_renderer_surface_state,
    },
    desktop::Window,
    reexports::wayland_server::{Resource, Weak, protocol::wl_surface::WlSurface},
    utils::{Physical, Rectangle},
    wayland::compositor::{get_children, get_parent},
};
use std::collections::HashMap;

/// Surface identities only: never retain a client buffer or wait for a commit.
#[derive(Default, Clone)]
pub(in crate::backend::tty::render::animation) struct Content {
    roots: Vec<Weak<WlSurface>>,
}

impl Content {
    pub(in crate::backend::tty::render::animation) fn capture(
        window: &Window,
        previous: Option<&Self>,
    ) -> Self {
        let Some(top) = window.toplevel() else {
            return Self::default();
        };
        let root = top.wl_surface();
        let geometry = window.geometry();
        // Keep identities through interrupted transitions, when the child may
        // already have resized ahead of the root and no longer match geometry.
        let mut roots = previous.map_or_else(Vec::new, |previous| previous.roots.clone());
        roots.retain(|weak| {
            weak.upgrade()
                .is_ok_and(|surface| get_parent(&surface).as_ref() == Some(root))
        });
        for surface in get_children(root)
            .into_iter()
            .filter(|surface| surface != root)
        {
            let fills_window = with_renderer_surface_state(&surface, |state| {
                state.buffer().is_some()
                    && state
                        .view()
                        .is_some_and(|view| Rectangle::new(view.offset, view.dst) == geometry)
            })
            .unwrap_or(false);
            let weak = surface.downgrade();
            if fills_window && !roots.contains(&weak) {
                roots.push(weak);
            }
        }
        Self { roots }
    }

    /// Give each member of a full-window subtree the same live reference box.
    /// Its size comes from the imported content, not the independently committed
    /// root geometry. This covers both child-first and root-first resizes.
    pub(super) fn references(
        &self,
        root: &WlSurface,
        surfaces: &[WaylandSurfaceRenderElement<GlesRenderer>],
        scale: f64,
    ) -> HashMap<Id, Rectangle<i32, Physical>> {
        let mut references = HashMap::new();
        for weak in &self.roots {
            let Ok(surface) = weak.upgrade() else {
                continue;
            };
            if get_parent(&surface).as_ref() != Some(root) {
                continue;
            }
            let id = Id::from_wayland_resource(&surface);
            let Some(element) = surfaces.iter().find(|element| element.id() == &id) else {
                continue;
            };
            let source = element.geometry(scale.into());
            if source.is_empty() {
                continue;
            }
            let mut pending = vec![surface];
            while let Some(surface) = pending.pop() {
                references.insert(Id::from_wayland_resource(&surface), source);
                pending.extend(
                    get_children(&surface)
                        .into_iter()
                        .filter(|child| child != &surface),
                );
            }
        }
        references
    }
}
