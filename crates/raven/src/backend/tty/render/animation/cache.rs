use super::super::SceneElement;
use super::{blend::Blend, snapshot::Snapshot};
use crate::{
    desktop::animation::{
        geometry::Geometry,
        timeline::{Sample, Timeline},
    },
    state::State,
};
use smithay::{
    backend::renderer::{
        ContextId, Renderer,
        element::Id,
        gles::{GlesRenderer, GlesTexProgram, GlesTexture},
        utils::CommitCounter,
    },
    desktop::Window,
    output::Output,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Rectangle, Serial, Transform},
};
use std::{
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

// Bound resources even if clients pile up blocked resizes or enormous frames.
pub(super) const MAX_SNAPSHOT_BYTES: usize = 128 * 1024 * 1024;
pub(super) const PENDING_LIFETIME: Duration = Duration::from_secs(2);

pub(super) struct Transition {
    pub(super) id: Id,
    pub(super) last_queued_image: Option<Blend>,
    pub(super) rendered_image: Option<Blend>,
    pub(super) serial: Serial,
    pub(super) snapshot: Rc<Snapshot>,
    pub(super) from: Geometry,
    pub(super) captured: Instant,
    pub(super) timeline: Option<Timeline>,
    pub(super) last_queued: Sample,
    pub(super) rendered: Sample,
}

#[derive(Default)]
pub(in crate::backend::tty) struct Animations {
    pub(super) entries: HashMap<WlSurface, Transition>,
    pub(super) output: Option<(Output, Rectangle<i32, Logical>, f64, Transform)>,
    pub(super) commit: CommitCounter,
    pub(super) now: Option<Instant>,
    pub(super) program: Option<Rc<GlesTexProgram>>,
    pub(super) context: Option<ContextId<GlesTexture>>,
}

impl Animations {
    pub fn clear(&mut self) {
        self.entries.clear();
        self.output = None;
        self.program = None;
        self.context = None;
    }
    pub fn remove(&mut self, root: &WlSurface) {
        self.entries.remove(root);
    }

    pub fn begin_frame(&mut self) {
        self.now = Some(Instant::now());
        self.commit.increment();
    }

    pub fn queued(&mut self) {
        for entry in self.entries.values_mut() {
            entry.last_queued = entry.rendered;
            entry.last_queued_image = entry.rendered_image.clone();
        }
    }

    /// No output damage means every animated draw was occluded. Do not keep
    /// requesting empty frames for an invisible transition. A later reveal uses
    /// the normal committed content and the ordinary visibility damage path.
    pub fn not_queued(&mut self) {
        self.entries.retain(|_, entry| entry.timeline.is_none());
    }

    pub fn active(&self) -> bool {
        self.entries
            .values()
            .any(|entry| entry.timeline.is_some() && entry.last_queued.progress < 1.0)
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.entries
            .values()
            .filter(|entry| entry.timeline.is_none())
            .map(|entry| entry.captured + PENDING_LIFETIME)
            .min()
    }

    pub(in crate::backend::tty::render) fn append(
        &mut self,
        renderer: &mut GlesRenderer,
        state: &State,
        window: &Window,
        area: Rectangle<i32, Logical>,
        scale: f64,
        active: bool,
        elements: &mut Vec<SceneElement>,
    ) -> bool {
        if self
            .context
            .as_ref()
            .is_some_and(|context| *context != renderer.context_id())
        {
            self.clear();
        }
        let Some(top) = window.toplevel() else {
            return false;
        };
        if !self.entries.contains_key(top.wl_surface()) {
            return false;
        }
        let available = MAX_SNAPSHOT_BYTES.saturating_sub(self.retained_bytes());
        let Some(entry) = self.entries.get_mut(top.wl_surface()) else {
            return false;
        };
        let Some(program) = self.program.clone() else {
            return false;
        };
        let sample = entry
            .timeline
            .as_ref()
            .map_or(entry.last_queued, |timeline| {
                timeline.sample(self.now.unwrap_or_else(Instant::now))
            });
        let visible = super::paint::physical(sample.geometry.frame, area, scale)
            .intersection(super::paint::physical(area, area, scale));
        if visible.is_none_or(|bounds| super::accounting::occluded(elements, bounds, scale)) {
            self.entries.remove(top.wl_surface());
            return false;
        }
        match super::frame::build(
            renderer,
            state,
            window,
            area,
            scale,
            active,
            entry,
            sample,
            self.commit,
            program,
            available,
        ) {
            Ok(image) => {
                entry.rendered = sample;
                entry.rendered_image = Some(image.clone());
                elements.push(SceneElement::ResizeBlend(image));
                true
            }
            Err(error) => {
                eprintln!("raven: resize animation cancelled: {error}");
                self.entries.remove(top.wl_surface());
                false
            }
        }
    }
}
