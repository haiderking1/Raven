use super::super::SceneElement;
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Frame, Offscreen, Renderer,
            element::utils::{Relocate, RelocateRenderElement},
            gles::{GlesError, GlesRenderer, GlesTexture},
            sync::SyncPoint,
            utils::draw_render_elements,
        },
    },
    utils::{Physical, Rectangle, Transform},
};
use std::cell::RefCell;

/// A compositor-owned copy, not an attachment to any wl_surface. The inputs
/// retain Smithay's Buffer references until the copy fence completes. This also
/// protects imported DMA-BUF textures when a replacement commit releases them.
pub(in crate::backend::tty::render) struct Snapshot {
    pub texture: GlesTexture,
    pub opaque: Vec<Rectangle<i32, Physical>>,
    pub bytes: usize,
    fence: SyncPoint,
    inputs: RefCell<Vec<SceneElement>>,
}

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("texture", &self.texture)
            .field("opaque", &self.opaque)
            .field("bytes", &self.bytes)
            .field("fence", &self.fence)
            .field("retained_inputs", &self.inputs.borrow().len())
            .finish()
    }
}

impl Snapshot {
    pub fn capture(
        renderer: &mut GlesRenderer,
        bounds: Rectangle<i32, Physical>,
        scale: f64,
        inputs: Vec<SceneElement>,
    ) -> Result<Self, GlesError> {
        let opaque = super::accounting::opaque_regions(&inputs, scale, bounds);
        let mut texture: GlesTexture =
            renderer.create_buffer(Fourcc::Abgr8888, (bounds.size.w, bounds.size.h).into())?;
        let shifted: Vec<_> = inputs
            .iter()
            .map(|element| {
                RelocateRenderElement::from_element(
                    element,
                    (-bounds.loc.x, -bounds.loc.y),
                    Relocate::Relative,
                )
            })
            .collect();
        let mut target = renderer.bind(&mut texture)?;
        let mut frame = renderer.render(&mut target, bounds.size, Transform::Normal)?;
        let damage = [Rectangle::from_size(bounds.size)];
        frame.clear([0.0, 0.0, 0.0, 0.0].into(), &damage)?;
        // The normal tree importer already resolved viewports, scale, transform,
        // root geometry offsets and synchronized subsurface placement.
        let drawn =
            draw_render_elements::<GlesRenderer, _, _>(&mut frame, scale, &shifted, &damage);
        let finished = frame.finish();
        drop(target);
        // Even a failed draw may have read some inputs. Do not release those
        // buffers until the submitted copy has finished.
        let fence = finished?;
        if let Err(error) = drawn {
            while fence.wait().is_err() {}
            return Err(error);
        }
        drop(shifted);
        Ok(Self {
            texture,
            opaque,
            bytes: bounds.size.w as usize * bounds.size.h as usize * 4,
            fence,
            inputs: RefCell::new(inputs),
        })
    }

    pub fn retained_bytes(&self, seen: &mut std::collections::HashSet<usize>) -> usize {
        if !seen.insert(self as *const Self as usize) {
            return 0;
        }
        self.retire_inputs();
        // During an interrupted copy, older snapshots can remain among the
        // fenced inputs. Count those too rather than bounding only map entries.
        self.bytes
            + self
                .inputs
                .borrow()
                .iter()
                .map(|element| match element {
                    SceneElement::ResizeBlend(group) => {
                        group.old.retained_bytes(seen) + group.current.retained_bytes(seen)
                    }
                    _ => 0,
                })
                .sum::<usize>()
    }

    pub fn retire_inputs(&self) {
        if self.fence.is_reached() {
            self.inputs.borrow_mut().clear();
        }
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        // Usually the existing output submission fence has already completed
        // this same-context copy. Cancellation before submission is the rare
        // path that must wait; no early wl_buffer.release is safe there.
        if !self.inputs.get_mut().is_empty() {
            while self.fence.wait().is_err() {}
        }
    }
}
