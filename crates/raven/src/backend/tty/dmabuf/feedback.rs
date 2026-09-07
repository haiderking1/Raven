use super::super::device::{Compositor, PlanePolicy};
use smithay::{
    backend::{
        drm::DrmNode,
        renderer::{ImportDma, gles::GlesRenderer},
    },
    reexports::wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1::TrancheFlags,
    wayland::dmabuf::{DmabufFeedback, DmabufFeedbackBuilder},
};
use std::io;

pub(in crate::backend::tty) struct Feedback {
    pub render: DmabufFeedback,
    pub scanout: DmabufFeedback,
}

impl Feedback {
    pub fn new(
        renderer: &GlesRenderer,
        compositor: &Compositor,
        render_node: DrmNode,
        kms_node: DrmNode,
        policy: PlanePolicy,
    ) -> io::Result<Option<Self>> {
        let imports = renderer.dmabuf_formats();
        if imports.iter().next().is_none() {
            return Ok(None);
        }
        let builder = DmabufFeedbackBuilder::new(render_node.dev_id(), imports.iter().copied());
        let render = builder.clone().build()?;
        // Only the selected primary plane is a client allocation target. Overlay
        // planes are removed at construction; the cursor copies into our own BO.
        // Preserve complete format/modifier pairs, including implicit modifiers.
        let primary = compositor
            .surface()
            .plane_info()
            .formats
            .iter()
            .copied()
            .filter(|format| {
                policy.primary
                    && imports.contains(format)
                    && format.code == compositor.format()
                    && compositor.modifiers().contains(&format.modifier)
            });
        let scanout = builder
            .add_preference_tranche(kms_node.dev_id(), Some(TrancheFlags::Scanout), primary)
            .build()?;
        Ok(Some(Self { render, scanout }))
    }
}
