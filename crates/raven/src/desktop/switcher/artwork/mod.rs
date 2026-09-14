#[cfg(test)]
mod companions_tests;
mod raster;
#[cfg(test)]
mod tests;
mod worker;
use super::layout::Layout;
use smithay::{
    backend::{allocator::Fourcc, renderer::element::memory::MemoryRenderBuffer},
    reexports::calloop::LoopSignal,
    utils::Transform,
};

pub(crate) struct Frame {
    pub buffer: MemoryRenderBuffer,
    pub layout: Layout,
}
#[derive(Clone)]
pub(super) struct Request {
    pub generation: u64,
    pub layout: Layout,
    pub scale: i32,
    pub selected: usize,
    pub apps: Vec<(String, String)>,
}
#[derive(Default)]
pub(crate) struct Artwork {
    generation: u64,
    worker: Option<worker::Worker>,
    pub frame: Option<Frame>,
    pub layout: Option<Layout>,
}
impl Artwork {
    pub fn request(
        &mut self,
        layout: Layout,
        scale: i32,
        selected: usize,
        apps: Vec<(String, String)>,
        signal: LoopSignal,
    ) {
        self.generation = self.generation.wrapping_add(1);
        if self
            .frame
            .as_ref()
            .is_some_and(|frame| frame.layout != layout)
        {
            self.frame = None;
        }
        self.layout = Some(layout.clone());
        let worker = self
            .worker
            .get_or_insert_with(|| worker::Worker::new(signal));
        worker.request(Request {
            generation: self.generation,
            layout,
            scale,
            selected,
            apps,
        });
    }
    pub fn poll(&mut self) -> Result<bool, String> {
        let Some(result) = self.worker.as_ref().and_then(|w| w.take()) else {
            return Ok(false);
        };
        if result.0.generation != self.generation || self.layout.is_none() {
            return Ok(false);
        }
        let pixels = result.1?;
        let request = result.0;
        let size = request.layout.rect.size;
        self.frame = Some(Frame {
            buffer: MemoryRenderBuffer::from_slice(
                &pixels,
                Fourcc::Argb8888,
                (size.w * request.scale, size.h * request.scale),
                request.scale,
                Transform::Normal,
                None,
            ),
            layout: request.layout,
        });
        Ok(true)
    }
    pub fn hide(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.frame = None;
        self.layout = None;
    }
}
