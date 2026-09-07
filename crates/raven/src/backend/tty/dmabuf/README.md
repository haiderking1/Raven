# Client GPU buffers

Linux DMA-BUF is advertised only with usable GLES texture-import formats. Global feedback describes the actual rendering device and import formats. Per-surface scanout advice intersects enabled primary-plane formats, selected swapchain formats and renderer import support. Render-only feedback remains the fallback.

Imported storage must pass EGL/GLES validation. An unset node hint is tagged only after successful import; a known different hint is not overwritten. Inactive or failed backends reject imports without using EGL. SHM remains available.

Every DMA-BUF attachment gets a fresh implicit acquire-readiness check before its commit applies. Calloop waits for plane writers without blocking the event thread. Surface destruction removes readiness sources; backend teardown removes all sources and globals before EGL/DRM.

Advice follows render results and is withdrawn from hidden, removed or no-longer-eligible surfaces. Copied hardware cursors receive render-only advice. Feedback maps reuse their allocations and keep weak surface references.

No explicit synchronization or multi-GPU capability is advertised. Direct scanout still requires runtime KMS acceptance; successful GPU import is not proof of scanout eligibility.
