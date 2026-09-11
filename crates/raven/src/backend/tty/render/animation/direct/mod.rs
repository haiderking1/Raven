mod content;
mod geometry;
pub(super) use content::Content;
mod paint;
pub(super) use paint::append;

use super::cache::Transition;
use crate::desktop::animation::{geometry::Geometry, timeline::Sample};
use smithay::{
    backend::renderer::element::Id,
    utils::{Physical, Rectangle, Serial},
};
use std::time::Instant;

pub(super) fn transition(
    serial: Serial,
    from: Geometry,
    bounds: Rectangle<i32, Physical>,
    live_content: Content,
) -> Transition {
    let sample = Sample {
        geometry: from,
        progress: 0.0,
    };
    Transition {
        id: Id::new(),
        snapshot: None,
        live_content,
        last_queued_image: None,
        rendered_image: None,
        serial,
        from,
        content_from: bounds,
        captured: Instant::now(),
        timeline: None,
        last_queued: sample,
        rendered: sample,
    }
}
