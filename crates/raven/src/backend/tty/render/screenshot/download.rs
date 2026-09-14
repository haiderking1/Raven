use crate::desktop::screenshot::encoding::Pixels;
use smithay::{
    backend::{
        allocator::Fourcc,
        egl::fence::EGLFence,
        renderer::{
            Bind, ExportMem,
            gles::{GlesMapping, GlesRenderer, GlesTexture},
            sync::SyncPoint,
        },
    },
    reexports::calloop::LoopSignal,
    utils::{Physical, Rectangle},
};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
pub(super) struct Download {
    pub generation: u64,
    mapping: GlesMapping,
    ready: Arc<AtomicU8>,
    rect: Rectangle<i32, Physical>,
}
impl Download {
    pub fn start(
        renderer: &mut GlesRenderer,
        texture: &mut GlesTexture,
        rect: Rectangle<i32, Physical>,
        generation: u64,
        signal: LoopSignal,
    ) -> Result<Self, String> {
        let target = renderer.bind(texture).map_err(|e| e.to_string())?;
        let region = Rectangle::new(
            (rect.loc.x, rect.loc.y).into(),
            (rect.size.w, rect.size.h).into(),
        );
        let mapping = renderer
            .copy_framebuffer(&target, region, Fourcc::Abgr8888)
            .map_err(|e| e.to_string())?;
        // The fence follows ReadPixels into the PBO, not just the scene draw.
        let fence =
            EGLFence::create(renderer.egl_context().display()).map_err(|e| e.to_string())?;
        renderer
            .with_context(|gl| unsafe {
                gl.Flush();
            })
            .map_err(|e| e.to_string())?;
        drop(target);
        let ready = Arc::new(AtomicU8::new(0));
        let done = ready.clone();
        std::thread::Builder::new()
            .name("raven-screenshot-fence".into())
            .spawn(move || {
                done.store(
                    if SyncPoint::from(fence).wait().is_ok() {
                        1
                    } else {
                        2
                    },
                    Ordering::Release,
                );
                signal.wakeup();
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            generation,
            mapping,
            ready,
            rect,
        })
    }
    pub fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire) != 0
    }
    pub fn finish(self, renderer: &mut GlesRenderer) -> Result<Pixels, String> {
        if self.ready.load(Ordering::Acquire) != 1 {
            return Err("screenshot GPU readback did not finish".into());
        }
        let bytes = renderer
            .map_texture(&self.mapping)
            .map_err(|e| e.to_string())?;
        Ok(Pixels {
            rgba: bytes.to_vec(),
            width: self.rect.size.w,
            height: self.rect.size.h,
        })
    }
}
