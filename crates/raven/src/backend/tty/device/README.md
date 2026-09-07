# Plane policy

Primary-plane scanout is enabled with matching output formats. Hardware cursors are implemented but opt-in because Smithay 0.7 contains unchecked cursor BO mapping and incomplete fast-copy padding clearing. Overlay planes and asynchronous presentation remain disabled.

- RAVEN_PRIMARY_SCANOUT=0 disables direct primary scanout.
- RAVEN_HARDWARE_CURSOR=1 opts into the pinned hardware-cursor path. Leave it unset for software cursors.

Both variables accept 0/false and 1/true. Invalid settings fail startup. These settings permit attempts, not guaranteed plane assignments. Check opt-in frame diagnostics for actual queued plane usage. A software cursor can prevent direct primary scanout; Raven never drops the cursor to make a window eligible.

The exporter admits successfully imported buffers with the compatible renderer-node hint. Smithay checks complete formats, modifiers, size, opacity, transforms and atomic plane compatibility. Every scene element remains available for GLES fallback.

Only a validated atomic plane-configuration rejection receives one forced full-composition retry. Optional planes then remain suspended until VT reactivation. Busy, inactive, disconnected, allocation and synchronization failures do not enter that retry. A failed replacement retains both error descriptions and stops the backend.
