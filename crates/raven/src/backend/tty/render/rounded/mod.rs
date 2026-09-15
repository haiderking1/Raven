mod cache;
mod element;
pub(in crate::backend::tty::render) mod program;
pub(in crate::backend::tty::render) mod shape;
use super::SceneElement;
/// Reference-matched design radius in logical pixels, independent of output DPI.
use crate::desktop::appearance::corners::RADIUS;
use crate::{desktop::animation::geometry::Geometry, state::State};
pub(super) use element::{Ring, Rounded};
use smithay::{
    backend::renderer::{
        element::Element,
        gles::{GlesError, GlesRenderer},
    },
    desktop::Window,
    utils::{Logical, Rectangle},
};
use std::{collections::HashSet, sync::Mutex};
pub(super) fn apply(
    renderer: &mut GlesRenderer,
    state: &State,
    window: &Window,
    geometry: Geometry,
    area: Rectangle<i32, Logical>,
    scale: f64,
    active: bool,
    start: usize,
    elements: &mut Vec<SceneElement>,
) -> Result<(), GlesError> {
    if state.applied_fullscreen_allocation(window).is_some() {
        return Ok(());
    }
    let programs = program::Programs::get(renderer)?;
    let physical = |rect: Rectangle<i32, Logical>| -> Rectangle<i32, smithay::utils::Physical> {
        Rectangle::from_extremities(
            (rect.loc - area.loc).to_physical_precise_round(scale),
            (rect.loc + rect.size - area.loc).to_physical_precise_round(scale),
        )
    };
    let frame = physical(geometry.frame);
    let client = physical(geometry.client);
    let width = (geometry.client.loc.x - geometry.frame.loc.x).max(0) as f32 * scale as f32;
    let outer = shape::Shape::new(frame, RADIUS * scale as f32);
    let inner = shape::Shape::new(client, (outer.radius - width).max(0.0));
    window
        .user_data()
        .insert_if_missing(|| Mutex::new(cache::Cache::default()));
    let mut cache = window
        .user_data()
        .get::<Mutex<cache::Cache>>()
        .unwrap()
        .lock()
        .unwrap();
    let body = elements.split_off(start);
    let mut ids = HashSet::new();
    if width > 0.0 {
        let color = state.appearance().border.premultiplied(active);
        let (id, commit) = cache.ring(outer, color, inner);
        elements.push(SceneElement::RoundedBorder(Ring {
            id,
            commit,
            shape: outer,
            color,
            inner,
            programs: programs.clone(),
        }));
    }
    for mut element in body {
        if matches!(element, SceneElement::Border(_)) {
            continue;
        }
        if let SceneElement::ResizeBlend(blend) = &mut element {
            blend.corners = Some(inner);
            elements.push(element);
            continue;
        }
        let id = element.id().clone();
        ids.insert(id.clone());
        let (commit, previous) = cache.surface(id, element.current_commit(), inner);
        elements.push(SceneElement::Rounded(Rounded {
            element: Box::new(element),
            shape: inner,
            commit,
            previous,
            scale,
            programs: programs.clone(),
        }));
    }
    cache.surfaces.retain(|id, _| ids.contains(id));
    Ok(())
}

#[cfg(test)]
mod tests;
