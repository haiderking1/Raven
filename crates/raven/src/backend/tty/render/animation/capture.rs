use super::super::windows;
use super::{
    cache::{Animations, MAX_SNAPSHOT_BYTES, Transition},
    content,
    snapshot::Snapshot,
};
use crate::{
    desktop::animation::{geometry::Geometry, timeline::Sample},
    state::State,
};
use smithay::backend::renderer::{Renderer, element::Id};
use smithay::{
    backend::renderer::gles::GlesRenderer, output::Output,
    reexports::wayland_server::protocol::wl_surface::WlSurface, utils::Serial,
};
use std::{error::Error, rc::Rc, time::Instant};

impl Animations {
    pub fn capture(
        &mut self,
        renderer: &mut GlesRenderer,
        state: &State,
        output: &Output,
        root: &WlSurface,
        serial: Serial,
    ) -> Result<bool, Box<dyn Error>> {
        let Some(area) = state.space().output_geometry(output) else {
            return Ok(false);
        };
        let scale = output.current_scale().fractional_scale();
        if self
            .context
            .as_ref()
            .is_some_and(|context| *context != renderer.context_id())
        {
            self.clear();
        }
        self.reconcile(state, output);
        self.context = Some(renderer.context_id());
        let Some(window) = state
            .visible_windows()
            .find(|w| w.toplevel().is_some_and(|top| top.wl_surface() == root))
        else {
            return Ok(false);
        };
        let Some(current) = Geometry::of(state, window) else {
            return Ok(false);
        };
        let direct = state.firefox_live_fullscreen_animation(window)
            || self
                .entries
                .get(root)
                .is_some_and(|entry| entry.snapshot.is_none());
        if let Some(entry) = self.entries.get_mut(root) {
            if entry.timeline.is_none() {
                // Keep the original geometry (and image for blended resizes)
                // across blocked commits while tracking the newest serial.
                entry.serial = serial;
                if direct {
                    entry.live_content =
                        super::direct::Content::capture(window, Some(&entry.live_content));
                    entry.snapshot = None;
                    entry.last_queued_image = None;
                    entry.rendered_image = None;
                }
                return Ok(true);
            }
        }
        let from = self
            .entries
            .get(root)
            .map_or(current, |entry| entry.last_queued.geometry);
        if direct {
            let bounds = super::paint::physical(from.client, area, scale);
            let live_content = super::direct::Content::capture(
                window,
                self.entries.get(root).map(|entry| &entry.live_content),
            );
            self.entries.insert(
                root.clone(),
                super::direct::transition(serial, from, bounds, live_content),
            );
            return Ok(true);
        }
        let Some(current_content) = content::bounds(state, window, area, scale) else {
            return Ok(false);
        };
        // Interrupted transitions start from the exact content rectangle that
        // was queued, independently of the surrounding frame animation.
        let bounds = self.entries.get(root).map_or(current_content, |entry| {
            entry
                .last_queued_image
                .as_ref()
                .map_or(entry.content_from, |image| image.bounds)
        });
        if bounds.is_empty() {
            return Ok(false);
        }
        let bytes = (bounds.size.w as usize)
            .checked_mul(bounds.size.h as usize)
            .and_then(|n| n.checked_mul(4));
        let retained = self.retained_bytes();
        if bytes.is_none_or(|bytes| bytes > MAX_SNAPSHOT_BYTES.saturating_sub(retained)) {
            self.remove(root);
            return Ok(false);
        }
        if self.program.is_none() {
            self.program = Some(super::blend::compile(renderer)?);
        }
        super::tree::validate(renderer, root)?;
        let mut elements = Vec::new();
        if let Some(entry) = self.entries.get(root) {
            // Capture the last actually queued blend and geometry, not either
            // endpoint and not a speculative wall-clock sample.
            let image = if let Some(image) = &entry.last_queued_image {
                image.clone()
            } else {
                super::frame::build(
                    renderer,
                    state,
                    window,
                    area,
                    scale,
                    entry,
                    entry.last_queued,
                    self.commit,
                    self.program
                        .as_ref()
                        .expect("compiled resize shader")
                        .clone(),
                    MAX_SNAPSHOT_BYTES.saturating_sub(retained),
                )?
            };
            elements.push(super::super::SceneElement::ResizeBlend(image));
        } else {
            windows::append_content(renderer, state, window, area, scale, &mut elements);
        }
        if elements.is_empty() {
            return Ok(false);
        }
        let snapshot = Rc::new(Snapshot::capture(renderer, bounds, scale, elements)?);
        let sample = Sample {
            geometry: from,
            progress: 0.0,
        };
        self.entries.insert(
            root.clone(),
            Transition {
                id: Id::new(),
                last_queued_image: None,
                rendered_image: None,
                serial,
                snapshot: Some(snapshot),
                live_content: super::direct::Content::default(),
                from,
                content_from: bounds,
                captured: Instant::now(),
                timeline: None,
                last_queued: sample,
                rendered: sample,
            },
        );
        Ok(true)
    }
}
