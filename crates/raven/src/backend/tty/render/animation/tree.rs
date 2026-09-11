use smithay::{
    backend::renderer::{
        element::{Kind, surface::WaylandSurfaceRenderElement},
        gles::GlesRenderer,
        utils::RendererSurfaceStateUserData,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{TraversalAction, with_surface_tree_downward},
};
use std::error::Error;

/// Normal rendering can skip an import failure. A persistent OLD snapshot must
/// instead fail as a whole, or it would freeze an incomplete subsurface tree.
pub(super) fn validate(
    renderer: &mut GlesRenderer,
    root: &WlSurface,
) -> Result<(), Box<dyn Error>> {
    let mut error: Option<Box<dyn Error>> = None;
    with_surface_tree_downward(
        root,
        (),
        |_, states, _| {
            let mapped = states
                .data_map
                .get::<RendererSurfaceStateUserData>()
                .is_some_and(|data| data.lock().unwrap().view().is_some());
            if mapped {
                TraversalAction::DoChildren(())
            } else {
                TraversalAction::SkipChildren
            }
        },
        |surface, states, _| {
            if error.is_some()
                || !states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .is_some_and(|data| data.lock().unwrap().view().is_some())
            {
                return;
            }
            match WaylandSurfaceRenderElement::from_surface(
                renderer,
                surface,
                states,
                (0.0, 0.0).into(),
                1.0,
                Kind::Unspecified,
            ) {
                Ok(Some(_)) => {}
                Ok(None) => error = Some("mapped snapshot surface has no imported buffer".into()),
                Err(cause) => error = Some(cause.into()),
            }
        },
        |_, _, _| true,
    );
    error.map_or(Ok(()), Err)
}
