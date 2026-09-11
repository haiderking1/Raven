use super::super::SceneElement;
use super::element::Animated;
use crate::state::State;
use smithay::{
    backend::renderer::{
        element::{
            Element, Kind,
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            utils::{CropRenderElement, Relocate, RelocateRenderElement, RescaleRenderElement},
        },
        gles::GlesRenderer,
        utils::CommitCounter,
    },
    desktop::Window,
    utils::{Logical, Physical, Rectangle, Scale},
};

pub(in crate::backend::tty::render) type LiveElement = Animated<
    CropRenderElement<
        RelocateRenderElement<
            RescaleRenderElement<CropRenderElement<WaylandSurfaceRenderElement<GlesRenderer>>>,
        >,
    >,
>;

pub(super) fn physical(
    rect: Rectangle<i32, Logical>,
    area: Rectangle<i32, Logical>,
    scale: f64,
) -> Rectangle<i32, Physical> {
    Rectangle::from_extremities(
        (rect.loc - area.loc).to_physical_precise_round(scale),
        (rect.loc + rect.size - area.loc).to_physical_precise_round(scale),
    )
}

pub(super) fn append_live(
    renderer: &mut GlesRenderer,
    state: &State,
    window: &Window,
    area: Rectangle<i32, Logical>,
    scale: f64,
    target: Rectangle<i32, Physical>,
    commit: CommitCounter,
    elements: &mut Vec<SceneElement>,
) {
    let output_clip = physical(area, area, scale);
    let Some(source) = super::content::bounds(state, window, area, scale) else {
        return;
    };
    if source.is_empty() || target.is_empty() {
        return;
    }
    let Some(clip) = target.intersection(output_clip) else {
        return;
    };
    let resize = Scale {
        x: f64::from(target.size.w) / f64::from(source.size.w),
        y: f64::from(target.size.h) / f64::from(source.size.h),
    };
    let Some(top) = window.toplevel() else {
        return;
    };
    let Some(origin) = state.window_surface_origin(window) else {
        return;
    };
    let surfaces: Vec<WaylandSurfaceRenderElement<GlesRenderer>> =
        render_elements_from_surface_tree(
            renderer,
            top.wl_surface(),
            (origin - area.loc).to_physical_precise_round(scale),
            scale,
            1.0,
            Kind::Unspecified,
        );
    for surface in surfaces {
        // Discard pixels outside committed XDG geometry before rescaling; an
        // early fullscreen-sized attachment is not the new window rectangle.
        let Some(surface) = CropRenderElement::from_element(surface, scale, source) else {
            continue;
        };
        let location = surface.geometry(scale.into()).loc;
        let opaque = surface
            .opaque_regions(scale.into())
            .into_iter()
            .map(|mut region| {
                region.loc += location;
                region
            })
            .collect();
        let scaled = RescaleRenderElement::from_element(surface, source.loc, resize);
        let moved = RelocateRenderElement::from_element(
            scaled,
            target.loc - source.loc,
            Relocate::Relative,
        );
        if let Some(element) = CropRenderElement::from_element(moved, scale, clip) {
            let opaque =
                super::opacity::project(opaque, source, target, element.geometry(scale.into()));
            elements.push(SceneElement::ResizeLive(Animated {
                element,
                commit,
                opaque,
            }));
        }
    }
}
