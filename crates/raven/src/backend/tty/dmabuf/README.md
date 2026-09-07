# Client GPU buffers

The TTY backend advertises Linux DMA-BUF only when GLES supplies usable texture-import formats. Default and surface feedback describe the actual rendering device and its import format/modifier pairs. The default main tranche permits composition, not direct scanout. SHM remains available.

Every imported buffer is validated by EGL/GLES. Unsupported formats and failed imports receive protocol failures. An inactive or failed backend does not attempt GPU imports. Startup reports whether DMA-BUF import was enabled.

Every DMA-BUF attachment installs a fresh implicit acquire-readiness blocker before Smithay applies its commit. Calloop waits for all plane writers without blocking the event thread. Reusing a wl_buffer repeats this check. Surface destruction removes its readiness sources; backend teardown removes all sources and the advertised global before dropping EGL/DRM.

This implements implicit synchronization. Raven does not advertise explicit synchronization, direct scanout, or multi-GPU support. Imported client buffers are GLES-composited into the output. Hardware driver behavior and application acceleration still require a real VT run.
