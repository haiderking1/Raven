use super::super::SceneElement;
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Frame, Offscreen, Renderer,
            element::{Element, Kind, RenderElement},
            gles::{GlesRenderer, GlesTexture},
            sync::SyncPoint,
        },
    },
    utils::{Physical, Rectangle, Scale, Size, Transform},
};
pub(super) struct Captured {
    pub image: GlesTexture,
    fence: SyncPoint,
    pub inputs: Vec<SceneElement>,
}
impl Captured {
    pub fn retire_inputs(&mut self) {
        if self.fence.is_reached() {
            self.inputs.clear();
        }
    }
}
impl Drop for Captured {
    fn drop(&mut self) {
        // A cancelled copy must not release client buffers before its own GPU
        // reads finish. This waits only that copy, never the entire GL context.
        if !self.inputs.is_empty() {
            while self.fence.wait().is_err() {}
        }
    }
}
pub(super) fn capture(
    renderer: &mut GlesRenderer,
    size: Size<i32, Physical>,
    scale: f64,
    elements: Vec<SceneElement>,
    clear: [f32; 4],
) -> Result<Captured, String> {
    let mut image: GlesTexture = renderer
        .create_buffer(Fourcc::Abgr8888, (size.w, size.h).into())
        .map_err(|e| e.to_string())?;
    // Capture upright output content, including overlays but not the pointer.
    let mut target = renderer.bind(&mut image).map_err(|e| e.to_string())?;
    let mut frame = renderer
        .render(&mut target, size, Transform::Normal)
        .map_err(|e| e.to_string())?;
    let output = Rectangle::from_size(size);
    frame
        .clear(clear.into(), &[output])
        .map_err(|e| e.to_string())?;
    let drawn = (|| -> Result<(), String> {
        for element in elements.iter().rev() {
            if element.kind() == Kind::Cursor {
                continue;
            }
            let geometry = element.geometry(Scale::from(scale));
            let Some(mut damage) = output.intersection(geometry) else {
                continue;
            };
            damage.loc -= geometry.loc;
            element
                .draw(&mut frame, element.src(), geometry, &[damage], &[])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    // The frozen image stays in the same GL stream. No CPU wait is needed to
    // display it; the returned fence is not a client/presentation owner.
    let fence = frame.finish().map_err(|e| e.to_string())?;
    drop(target);
    if let Err(error) = drawn {
        while fence.wait().is_err() {}
        return Err(error);
    }
    Ok(Captured {
        image,
        fence,
        inputs: elements,
    })
}
