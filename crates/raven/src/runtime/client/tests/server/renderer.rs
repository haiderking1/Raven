use smithay::backend::{
    egl::{EGLContext, EGLDevice, EGLDisplay},
    renderer::gles::GlesRenderer,
};
use std::error::Error;

pub(super) fn create() -> Result<GlesRenderer, Box<dyn Error>> {
    let device = EGLDevice::enumerate()?
        .next()
        .ok_or("no EGL device available")?;
    // The test owns this device display and context. No DRM master or session backend.
    let display = unsafe { EGLDisplay::new(device) }?;
    let context = EGLContext::new(&display)?;
    Ok(unsafe { GlesRenderer::new(context) }?)
}
