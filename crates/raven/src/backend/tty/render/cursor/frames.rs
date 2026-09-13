use smithay::{
    backend::{allocator::Fourcc, renderer::element::memory::MemoryRenderBuffer},
    utils::{Logical, Point, Rectangle, Size, Transform},
};
use std::{io, time::Duration};
use xcursor::parser::Image;

pub(super) struct Frame {
    pub buffer: MemoryRenderBuffer,
    pub size: Size<i32, Logical>,
    pub source: Rectangle<f64, Logical>,
    pub hotspot: Point<f64, Logical>,
    pub delay: Duration,
}

pub(super) struct Animation {
    pub frames: Vec<Frame>,
    duration: Duration,
}

impl Animation {
    pub fn new(images: Vec<Image>, target: u32, logical_size: u32) -> io::Result<Self> {
        let nominal = images
            .iter()
            .filter(|image| image.size > 0)
            .min_by_key(|image| (image.size.abs_diff(target), std::cmp::Reverse(image.size)))
            .map(|image| image.size)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "cursor has no sized images")
            })?;
        let ratio = f64::from(logical_size) / f64::from(nominal);
        let mut frames = Vec::new();
        let mut duration = Duration::ZERO;
        for image in images.into_iter().filter(|image| image.size == nominal) {
            let width = (f64::from(image.width) * ratio).round().max(1.0);
            let height = (f64::from(image.height) * ratio).round().max(1.0);
            if width > f64::from(i32::MAX) || height > f64::from(i32::MAX) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "cursor geometry exceeds i32",
                ));
            }
            // The parser's pixels_rgba field contains the file's little-endian
            // ARGB words. Convert word endianness, not channel order.
            let mut pixels = image.pixels_rgba;
            for pixel in pixels.chunks_exact_mut(4) {
                let native =
                    u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]).to_ne_bytes();
                pixel.copy_from_slice(&native);
            }
            let buffer = MemoryRenderBuffer::from_slice(
                &pixels,
                Fourcc::Argb8888,
                (image.width as i32, image.height as i32),
                1,
                Transform::Normal,
                None,
            );
            // Zero-delay frames must still advance without an immediate timer loop.
            let delay = Duration::from_millis(u64::from(image.delay.max(1)));
            duration += delay;
            frames.push(Frame {
                buffer,
                source: Rectangle::from_size(
                    (f64::from(image.width), f64::from(image.height)).into(),
                ),
                size: (width as i32, height as i32).into(),
                hotspot: (
                    f64::from(image.xhot) * width / f64::from(image.width),
                    f64::from(image.yhot) * height / f64::from(image.height),
                )
                    .into(),
                delay,
            });
        }
        Ok(Self { frames, duration })
    }

    /// Advance by elapsed time, skipping missed frames rather than queuing them.
    pub fn frame_at(&self, elapsed: Duration) -> (&Frame, Option<Duration>) {
        if self.frames.len() == 1 {
            return (&self.frames[0], None);
        }
        let mut position = elapsed.as_nanos() % self.duration.as_nanos();
        for frame in &self.frames {
            let delay = frame.delay.as_nanos();
            if position < delay {
                return (frame, Some(Duration::from_nanos((delay - position) as u64)));
            }
            position -= delay;
        }
        unreachable!("animation position is within its cycle")
    }
}
