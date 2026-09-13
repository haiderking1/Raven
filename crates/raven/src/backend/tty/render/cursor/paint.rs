use super::Cursors;
use smithay::{
    backend::renderer::{
        element::{Kind, memory::MemoryRenderBufferRenderElement},
        gles::{GlesError, GlesRenderer},
    },
    input::pointer::CursorIcon,
    utils::{Logical, Point},
};
use std::time::Instant;

impl Cursors {
    pub fn render(
        &mut self,
        renderer: &mut GlesRenderer,
        icon: CursorIcon,
        pointer: Point<f64, Logical>,
        scale: f64,
    ) -> Result<MemoryRenderBufferRenderElement<GlesRenderer>, GlesError> {
        let target = (f64::from(self.size) * scale).ceil().max(1.0) as u32;
        let now = Instant::now();
        let start = match self.active {
            Some((previous, previous_target, start))
                if previous == icon && previous_target == target =>
            {
                start
            }
            _ => {
                self.active = Some((icon, target, now));
                now
            }
        };
        let animation = self.animation(icon, target);
        let (frame, delay) = animation.frame_at(now.duration_since(start));
        self.deadline = delay.map(|delay| now + delay);
        MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            (pointer - frame.hotspot).to_physical(scale),
            &frame.buffer,
            None,
            Some(frame.source),
            Some(frame.size),
            Kind::Cursor,
        )
    }
}
