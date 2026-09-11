use super::super::SceneElement;
use crate::{desktop::animation::participation::GroupParticipation, state::State};
use smithay::{
    backend::renderer::element::{Element, Id},
    output::Output,
    utils::{Physical, Rectangle},
};

type Rect = Rectangle<i32, Physical>;

fn add_opaque(opaque: &mut Vec<Rect>, rect: Rect) {
    opaque.extend(rect.subtract_rects(opaque.iter().copied()));
}

pub(super) fn occluded(elements: &[SceneElement], bounds: Rect, scale: f64) -> bool {
    let opaque = elements.iter().flat_map(|element| {
        let location = element.geometry(scale.into()).loc;
        element
            .opaque_regions(scale.into())
            .into_iter()
            .map(move |mut region| {
                region.loc += location;
                region
            })
    });
    bounds.subtract_rects(opaque).is_empty()
}

/// Internal front-to-back visibility of the live image before it is mixed.
pub(super) fn live_regions(elements: &[SceneElement], scale: f64) -> Vec<(Id, Vec<Rect>)> {
    let mut opaque = Vec::new();
    let mut live = Vec::new();
    for element in elements {
        let geometry = element.geometry(scale.into());
        if matches!(element, SceneElement::ResizeLive(_)) && element.alpha() > 0.0 {
            live.push((
                element.id().clone(),
                geometry.subtract_rects(opaque.iter().copied()),
            ));
        }
        for mut region in element.opaque_regions(scale.into()) {
            region.loc += geometry.loc;
            add_opaque(&mut opaque, region);
        }
    }
    live
}

pub(super) fn opaque_regions(elements: &[SceneElement], scale: f64, bounds: Rect) -> Vec<Rect> {
    let mut opaque = Vec::new();
    for element in elements {
        for mut region in element.opaque_regions(scale.into()) {
            region.loc += element.geometry(scale.into()).loc;
            if let Some(mut region) = region.intersection(bounds) {
                region.loc -= bounds.loc;
                add_opaque(&mut opaque, region);
            }
        }
    }
    opaque
}

/// Extend output participation only for live pixels actually sampled by a group.
/// The DRM result remains the authority on whether that group was rendered at all.
pub(in crate::backend::tty::render) fn record(
    state: &State,
    output: &Output,
    elements: &[SceneElement],
) {
    let mut groups = state.animations.participation.borrow_mut();
    groups.clear();
    let Some(area) = state.space().output_geometry(output) else {
        return;
    };
    let scale = output.current_scale().fractional_scale();
    let clip = super::paint::physical(area, area, scale);
    let mut opaque = Vec::new();
    for element in elements {
        if let SceneElement::ResizeBlend(group) = element {
            let live = group
                .live
                .iter()
                .map(|(id, regions)| {
                    let area = regions
                        .iter()
                        .filter_map(|region| region.intersection(clip))
                        .flat_map(|region| region.subtract_rects(opaque.iter().copied()))
                        .map(|region| region.size.w as usize * region.size.h as usize)
                        .sum();
                    (id.clone(), area)
                })
                .collect();
            groups.push(GroupParticipation {
                group: group.id.clone(),
                live,
            });
        }
        let geometry = element.geometry(scale.into());
        for mut region in element.opaque_regions(scale.into()) {
            region.loc += geometry.loc;
            if let Some(region) = region.intersection(clip) {
                add_opaque(&mut opaque, region);
            }
        }
    }
}
