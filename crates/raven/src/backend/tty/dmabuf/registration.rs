use super::{super::device::Device, pending::Pending};
use crate::state::State;
use smithay::{
    backend::renderer::ImportDma,
    reexports::{calloop::LoopHandle, wayland_server::DisplayHandle},
    wayland::dmabuf::{DmabufGlobal, DmabufState},
};
use std::io;

/// Own the delegate and global together so failed installs and backend teardown
/// remove the advertisement before dropping the renderer that validates it.
pub(in crate::backend::tty) struct Registration {
    pub state: DmabufState,
    pub global: DmabufGlobal,
    pub(super) pending: Pending,
    display: DisplayHandle,
    disabled: bool,
}

impl Registration {
    pub fn new(
        device: &Device,
        display: DisplayHandle,
        handle: LoopHandle<'static, State>,
    ) -> io::Result<Option<Self>> {
        // Texture import formats include external-only formats. KMS and GLES
        // render-target formats would incorrectly exclude usable client buffers.
        let formats = device.renderer.dmabuf_formats();
        let Some(feedback) = device.feedback.as_ref() else {
            eprintln!("raven: no GLES DMA-BUF import formats; clients remain SHM-only");
            return Ok(None);
        };
        // Advertise the rendering device, not the connector or scanout target.
        let node = device.render_node;
        let mut state = DmabufState::new();
        // Smithay 0.7 advertises version 5 with the v4 feedback mechanism.
        // The sole main tranche describes composition imports, without Scanout.
        let global = state.create_global_with_default_feedback::<State>(&display, &feedback.render);
        eprintln!(
            "raven: Linux DMA-BUF import enabled on {node}, {} format/modifier pairs",
            formats.iter().count()
        );
        Ok(Some(Self {
            state,
            global,
            pending: Pending::new(handle),
            display,
            disabled: false,
        }))
    }

    pub fn disable(&mut self) {
        if !self.disabled {
            self.state
                .disable_global::<State>(&self.display, &self.global);
            self.disabled = true;
        }
        self.pending.clear();
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        self.disable();
        self.state
            .destroy_global::<State>(&self.display, self.global);
    }
}
