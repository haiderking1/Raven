use super::super::{SceneElement, borders};
use super::element::Animated;
use crate::{
    desktop::animation::{geometry::Geometry, timeline::Sample},
    state::State,
};
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
        RelocateRenderElement<RescaleRenderElement<WaylandSurfaceRenderElement<GlesRenderer>>>,
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
    active: bool,
    sample: Sample,
    commit: CommitCounter,
    elements: &mut Vec<SceneElement>,
) {
    let output_clip = physical(area, area, scale);
    borders::append_at(
        state,
        window,
        area,
        scale,
        active,
        sample.geometry.frame,
        sample.geometry.client,
        1.0,
        elements,
    );
    let Some(current) = Geometry::of(state, window) else {
        return;
    };
    let Some(top) = window.toplevel() else {
        return;
    };
    let Some(origin) = state.window_surface_origin(window) else {
        return;
    };
    let source = physical(current.client, area, scale);
    let target = physical(sample.geometry.client, area, scale);
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
