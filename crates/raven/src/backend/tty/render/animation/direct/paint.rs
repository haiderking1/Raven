use super::super::{element::Animated, opacity, paint::physical};
use crate::{backend::tty::render::SceneElement, state::State};
use smithay::{
    backend::renderer::{
        element::{
            Element, Id, Kind,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            utils::{CropRenderElement, Relocate, RelocateRenderElement, RescaleRenderElement},
        },
        gles::GlesRenderer,
        utils::CommitCounter,
    },
    desktop::Window,
    utils::{Logical, Rectangle, Scale},
};

/// Draw client buffers directly, retaining their identity, fences and feedback.
/// Animated/tiled mapping transforms the surface tree into the client rectangle.
/// Floating interactive mapping instead preserves child offsets and natural
/// extents, squeezing only overflow as in Hyprland. Neither retains an image.
pub(in crate::backend::tty::render::animation) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    window: &Window,
    area: Rectangle<i32, Logical>,
    scale: f64,
    client: Rectangle<i32, Logical>,
    commit: CommitCounter,
    content: &super::Content,
    floating_resize: bool,
) -> Vec<SceneElement> {
    let Some(top) = window.toplevel() else {
        return Vec::new();
    };
    let Some(origin) = state.window_surface_origin(window) else {
        return Vec::new();
    };
    let geometry = window.geometry();
    let source_window = physical(
        Rectangle::new(origin + geometry.loc, geometry.size),
        area,
        scale,
    );
    let target = physical(client, area, scale);
    let Some(clip) = target.intersection(physical(area, area, scale)) else {
        return Vec::new();
    };
    if source_window.is_empty() || target.is_empty() {
        return Vec::new();
    }
    let root_id = Id::from_wayland_resource(top.wl_surface());
    let surfaces: Vec<WaylandSurfaceRenderElement<GlesRenderer>> =
        render_elements_from_surface_tree(
            renderer,
            top.wl_surface(),
            (origin - area.loc).to_physical_precise_round(scale),
            scale,
            1.0,
            Kind::Unspecified,
        );
    let references = content.references(top.wl_surface(), &surfaces, scale);
    let native = floating_resize.then(|| super::sampling::native_surfaces(window, scale));
    let elements: Vec<_> = surfaces
        .into_iter()
        .filter_map(|surface| {
            let main = surface.id() == &root_id;
            let source = if main && floating_resize {
                source_window.intersection(surface.geometry(scale.into()))?
            } else if main {
                source_window
            } else {
                surface.geometry(scale.into())
            };
            if source.is_empty() {
                return None;
            }
            // Full-window GTK content may resize before or after its root.
            // Map that subtree from its own current bounds, not stale root size.
            // Other surfaces retain the ordinary root-relative transform.
            let reference = references
                .get(surface.id())
                .copied()
                .unwrap_or(source_window);
            let destination = if floating_resize {
                let buffer = surface.geometry(scale.into());
                // Hyprland compares against the size acknowledged with this
                // commit, not the newer pointer target. Ordinary roots still
                // fill the target; genuinely undersized roots stay natural.
                let reported = top.current_state().size.unwrap_or(geometry.size);
                let small = buffer.size.w as f64 + scale < reported.w as f64 * scale
                    || buffer.size.h as f64 + scale < reported.h as f64 * scale;
                super::floating::destination(source, source_window, target, main, small)
            } else {
                super::geometry::project(source, reference, target)
            };
            if destination.is_empty() {
                return None;
            }
            let native = native
                .as_ref()
                .is_some_and(|native| native.contains(surface.id()));
            let surface = CropRenderElement::from_element(surface, scale, source)?;
            let original_source = surface.src();
            let transform = surface.transform();
            let location = surface.geometry(scale.into()).loc;
            let opaque = surface
                .opaque_regions(scale.into())
                .into_iter()
                .map(|mut rect| {
                    rect.loc += location;
                    rect
                })
                .collect();
            let resized = RescaleRenderElement::from_element(
                surface,
                source.loc,
                Scale {
                    x: f64::from(destination.size.w) / f64::from(source.size.w),
                    y: f64::from(destination.size.h) / f64::from(source.size.h),
                },
            );
            let moved = RelocateRenderElement::from_element(
                resized,
                destination.loc - source.loc,
                Relocate::Relative,
            );
            let element = CropRenderElement::from_element(moved, scale, clip)?;
            let visible = element.geometry(scale.into());
            let sampling = native.then(|| {
                super::sampling::source(original_source, source, destination, visible, transform)
            });
            // Only the original opaque pixels are guaranteed opaque. Extended
            // edge coverage is deliberately not used to occlude lower windows.
            let opaque_target = if native {
                Rectangle::new(destination.loc, source.size)
            } else {
                destination
            };
            let opaque = opacity::project(opaque, source, opaque_target, visible);
            Some(SceneElement::ResizeLive(Animated {
                element,
                commit,
                opaque,
                source: sampling,
            }))
        })
        .collect();
    elements
}
