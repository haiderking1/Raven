use smithay::{
    backend::renderer::{element::Id, utils::with_renderer_surface_state},
    desktop::Window,
    utils::{Buffer, Physical, Rectangle, Transform},
    wayland::{
        compositor::{get_children, with_states},
        viewporter::ViewportCachedState,
    },
};
use std::collections::HashSet;

/// Match Hyprland's expected-size and scale-awareness checks. A child without
/// a viewport has no independent expected size in that path. Legacy clients
/// using integer buffer scaling on fractional outputs retain ordinary scaling.
pub(super) fn native_surfaces(window: &Window, scale: f64) -> HashSet<Id> {
    let mut native = HashSet::new();
    let Some(top) = window.toplevel() else {
        return native;
    };
    let root = top.wl_surface();
    let mut pending = vec![root.clone()];
    while let Some(surface) = pending.pop() {
        pending.extend(
            get_children(&surface)
                .into_iter()
                .filter(|child| child != &surface),
        );
        let viewport = with_states(&surface, |states| {
            *states.cached_state.get::<ViewportCachedState>().current()
        });
        if &surface != root && viewport.src.is_none() && viewport.dst.is_none() {
            continue;
        }
        let aware = scale.ceil() == 1.0
            || with_renderer_surface_state(&surface, |state| {
                f64::from(state.buffer_scale()) != scale.ceil() && viewport.dst.is_some()
            })
            .unwrap_or(false);
        if aware {
            native.insert(Id::from_wayland_resource(&surface));
        }
    }
    native
}

/// Keep the original texel density while changing the destination rectangle.
/// Use the same transform/viewport conversion as Smithay's crop element, but
/// relative to the natural extent rather than the resized destination. Source
/// coordinates may extend outside the texture: GLES clamps those to edge texels.
/// The visible rectangle includes output clipping, including left/top clipping.
pub(super) fn source(
    original: Rectangle<f64, Buffer>,
    natural: Rectangle<i32, Physical>,
    destination: Rectangle<i32, Physical>,
    visible: Rectangle<i32, Physical>,
    transform: Transform,
) -> Rectangle<f64, Buffer> {
    let ratio = original.size / transform.invert().transform_size(natural.size).to_f64();
    let relative = Rectangle::new(visible.loc - destination.loc, visible.size);
    let mut source = relative.to_f64().to_logical(1.0).to_buffer(
        ratio,
        transform,
        &natural.size.to_f64().to_logical(1.0),
    );
    source.loc += original.loc;
    source
}
