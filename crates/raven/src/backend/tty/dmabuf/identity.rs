use smithay::backend::{
    drm::{DrmDevice, DrmNode, NodeType},
    egl::EGLDevice,
    renderer::gles::GlesRenderer,
};
use std::io;

pub(in crate::backend::tty) fn renderer_node(
    renderer: &GlesRenderer,
    drm: &DrmDevice,
) -> io::Result<DrmNode> {
    // Query this renderer's EGL display, never the system's default GPU.
    if let Ok(egl_device) = EGLDevice::device_for_display(renderer.egl_context().display()) {
        if let Ok(Some(node)) = egl_device.try_get_render_node() {
            return Ok(node);
        }
    }
    // The EGL display was constructed from GBM on this DRM fd. Older EGL
    // implementations may lack device-query extensions. Prefer the matching
    // render node, which clients can open without DRM master authentication.
    let node = DrmNode::from_file(drm.device_fd()).map_err(io::Error::other)?;
    match node.node_with_type(NodeType::Render) {
        Some(render) => render.map_err(io::Error::other),
        None => Ok(node),
    }
}
