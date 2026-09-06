mod selection;

use super::session::SessionDevice;
use smithay::{
    backend::{
        allocator::{
            Fourcc,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        },
        drm::{
            DrmDevice, DrmDeviceFd, DrmDeviceNotifier, compositor::DrmCompositor,
            exporter::gbm::GbmFramebufferExporter,
        },
        egl::{EGLContext, EGLDisplay},
        renderer::gles::GlesRenderer,
        session::libseat::LibSeatSession,
        udev::UdevBackend,
    },
    output::{Mode as OutputMode, Output, PhysicalProperties},
    reexports::drm::control::{Device as _, Mode, connector, crtc},
    utils::{DeviceFd, Transform},
};
use std::{error::Error, io::ErrorKind, path::Path};

type Compositor =
    DrmCompositor<GbmAllocator<DrmDeviceFd>, GbmFramebufferExporter<DrmDeviceFd>, (), DrmDeviceFd>;

/// Fields drop in declaration order: framebuffer users, renderer, DRM, then libseat fd.
/// The event-loop DRM notifier must be removed before dropping this object.
pub(super) struct Device {
    pub compositor: Compositor,
    pub renderer: GlesRenderer,
    pub drm: DrmDevice,
    pub output: Output,
    pub connector: connector::Handle,
    pub crtc: crtc::Handle,
    pub mode: Mode,
    _session_device: SessionDevice,
}

impl Device {
    pub fn probe(
        session: &mut LibSeatSession,
        udev: &UdevBackend,
    ) -> Result<(Self, DrmDeviceNotifier), Box<dyn Error>> {
        let mut paths: Vec<_> = udev
            .device_list()
            .map(|(_, path)| path.to_owned())
            .collect();
        paths.sort();
        let mut failures = Vec::new();
        for path in paths {
            match Self::open(session, &path) {
                Ok(device) => return Ok(device),
                Err(error) => failures.push(format!("{}: {error}", path.display())),
            }
        }
        Err(format!(
            "no connected KMS output with a compatible GBM/GLES configuration on this seat; {}",
            if failures.is_empty() {
                "udev found no seat GPUs".into()
            } else {
                failures.join("; ")
            }
        )
        .into())
    }

    fn open(
        session: &mut LibSeatSession,
        path: &Path,
    ) -> Result<(Self, DrmDeviceNotifier), Box<dyn Error>> {
        let session_device = SessionDevice::open(session, path)?;
        let fd = DrmDeviceFd::new(DeviceFd::from(session_device.duplicate()?));
        let (mut drm, notifier) = DrmDevice::new(fd.clone(), true)?;
        let connectors = selection::connected(&drm)?;
        if connectors.is_empty() {
            return Err("no connected connectors with modes".into());
        }
        let gbm = GbmDevice::new(fd)?;
        // SAFETY: this is a GBM native display, not a guessed raw pointer. EGLDisplay
        // owns a clone of the GBM device, retaining its fd for the display's lifetime.
        let display = unsafe { EGLDisplay::new(gbm.clone())? };
        let context = EGLContext::new(&display)?;
        // SAFETY: this freshly created context is not current on any other thread.
        // The renderer and all its use remain on the calloop thread.
        let renderer = unsafe { GlesRenderer::new(context)? };
        let formats = renderer.egl_context().dmabuf_render_formats().clone();
        let mut failures = Vec::new();
        for connector in connectors {
            for crtc in connector.crtcs {
                for mode in &connector.modes {
                    let attempt = (|| -> Result<_, Box<dyn Error>> {
                        let surface =
                            drm.create_surface(crtc, *mode, &[connector.info.handle()])?;
                        let size = connector.info.size().unwrap_or((0, 0));
                        let output = Output::new(
                            format!(
                                "{:?}-{}",
                                connector.info.interface(),
                                connector.info.interface_id()
                            ),
                            PhysicalProperties {
                                size: (size.0 as i32, size.1 as i32).into(),
                                subpixel: connector.info.subpixel().into(),
                                make: "Unknown".into(),
                                model: "KMS display".into(),
                            },
                        );
                        let output_mode = OutputMode::from(*mode);
                        output.change_current_state(
                            Some(output_mode),
                            Some(Transform::Normal),
                            None,
                            Some((0, 0).into()),
                        );
                        output.set_preferred(output_mode);
                        let allocator = GbmAllocator::new(
                            gbm.clone(),
                            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
                        );
                        let compositor = Compositor::new(
                            &output,
                            surface,
                            None,
                            allocator,
                            GbmFramebufferExporter::new(gbm.clone(), None),
                            [Fourcc::Xrgb8888, Fourcc::Argb8888],
                            formats.iter().copied(),
                            drm.cursor_size(),
                            None,
                        )?;
                        Ok((compositor, output))
                    })();
                    match attempt {
                        Ok((compositor, output)) => {
                            eprintln!(
                                "raven: using {} on {} at {}x{}",
                                output.name(),
                                path.display(),
                                mode.size().0,
                                mode.size().1
                            );
                            return Ok((
                                Self {
                                    compositor,
                                    renderer,
                                    drm,
                                    output,
                                    connector: connector.info.handle(),
                                    crtc,
                                    mode: *mode,
                                    _session_device: session_device,
                                },
                                notifier,
                            ));
                        }
                        Err(error) => failures.push(format!(
                            "{:?}/{crtc:?} {}x{}: {error}",
                            connector.info.handle(),
                            mode.size().0,
                            mode.size().1
                        )),
                    }
                }
            }
        }
        Err(format!(
            "all connector/CRTC/mode configurations failed: {}",
            failures.join("; ")
        )
        .into())
    }

    pub fn validate_output(&self) -> Result<(), Box<dyn Error>> {
        let info = self.drm.get_connector(self.connector, true)?;
        if info.state() != connector::State::Connected || !info.modes().contains(&self.mode) {
            return Err("the selected output was disconnected or its mode disappeared; restart Raven to select another output".into());
        }
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), Box<dyn Error>> {
        self.drm.activate(true)?;
        self.validate_output()?;
        // activate(true) synchronously disables the CRTCs. reset_state alone
        // does not release DrmCompositor's pending/queued frame; clear also
        // releases those references before we render another frame.
        self.compositor.clear()?;
        self.compositor.reset_state()?;
        self.compositor.reset_buffers();
        // Discard pre-pause pageflips only after synchronous CRTC disable. Otherwise
        // a stale flip could complete the first new frame's bookkeeping.
        loop {
            match self.drm.receive_events() {
                Ok(events) => {
                    if events.count() == 0 {
                        break;
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}
