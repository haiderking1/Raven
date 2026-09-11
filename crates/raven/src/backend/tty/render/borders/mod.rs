mod cache;
mod element;
mod focus;
use super::SceneElement;
use crate::state::State;
use cache::BorderCache;
pub(super) use element::BorderElement;
pub(super) use focus::active_root;
use smithay::{
    desktop::Window,
    utils::{Logical, Physical, Rectangle},
};
use std::sync::Mutex;

pub(super) fn append(
    state: &State,
    window: &Window,
    output: Rectangle<i32, Logical>,
    scale: f64,
    active: bool,
    elements: &mut Vec<SceneElement>,
) {
    let Some(frame) = state.window_frame_geometry(window) else {
        return;
    };
    let Some(client) = state.window_client_geometry(window) else {
        return;
    };
    append_at(
        state, window, output, scale, active, frame, client, 1.0, elements,
    );
}

/// Paint the visual frame and client edges without changing layout geometry.
pub(super) fn append_at(
    state: &State,
    window: &Window,
    output: Rectangle<i32, Logical>,
    scale: f64,
    active: bool,
    frame: Rectangle<i32, Logical>,
    client: Rectangle<i32, Logical>,
    alpha: f32,
    elements: &mut Vec<SceneElement>,
) {
    if frame == client || alpha <= 0.0 {
        return;
    }
    let physical = |mut rect: Rectangle<i32, Logical>| -> Rectangle<i32, Physical> {
        rect.loc -= output.loc;
        Rectangle::from_extremities(
            rect.loc.to_physical_precise_round(scale),
            (rect.loc + rect.size).to_physical_precise_round(scale),
        )
    };
    let frame = physical(frame);
    let client = physical(client);
    let clip = physical(output);
    let end = frame.loc + frame.size;
    let inner_end = client.loc + client.size;
    // Top/bottom own the corners, so translucent edges never double-blend.
    let stripes = [
        Rectangle::from_extremities(frame.loc, (end.x, client.loc.y)),
        Rectangle::from_extremities((frame.loc.x, inner_end.y), end),
        Rectangle::from_extremities((frame.loc.x, client.loc.y), (client.loc.x, inner_end.y)),
        Rectangle::from_extremities((inner_end.x, client.loc.y), (end.x, inner_end.y)),
    ];
    let color = state
        .appearance()
        .border
        .premultiplied(active)
        .map(|channel| channel * alpha);
    // The cache belongs to the Window, not a map retaining dead clients. It
    // contains only compositor solids, never surface buffers or textures.
    window
        .user_data()
        .insert_if_missing(|| Mutex::new(BorderCache::default()));
    let mut cache = window
        .user_data()
        .get::<Mutex<BorderCache>>()
        .unwrap()
        .lock()
        .unwrap();
    for (stripe, geometry) in cache.stripes.iter_mut().zip(stripes) {
        if let Some(geometry) = geometry
            .intersection(clip)
            .filter(|g| g.size.w > 0 && g.size.h > 0)
        {
            elements.push(SceneElement::Border(stripe.element(geometry, color)));
        }
    }
}
