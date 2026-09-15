# Rounded window rendering

Normal window frames use a 16-logical-pixel radius, scaled with the output and
clamped for small windows. This is a visual approximation of the supplied
reference, not an Apple API constant. Existing border colors, widths, gaps, and
client decoration negotiation remain unchanged. No titlebar buttons are added.

The final window body is masked, not the individual buffer's corners. Root and
subsurface draws retain their element identity, texture sampling, acquire-fence
handling, and buffer ownership. Masked elements cannot advertise direct scanout.
Their opaque regions exclude the corner cutouts. Window-local commit caches keep
idle scenes undamaged and preserve client damage when only the source changes.

Masks use framebuffer coordinates derived from the frame projection, so output
rotation, reflection, cropping, and screenshot relocation do not rotate the shape
away from its window. The border uses independently rounded inner and outer
rectangles to avoid fractional-scale gaps. Shader objects belong to the EGL
context; no framebuffer copy or extra rendering timer is introduced.

Resize snapshots remain unmasked internally. The displayed live or blended frame
gets the current animated shape after composition. Drag previews use their moved
allocation. Popups stay outside this masking pass; displayed fullscreen owners
remain square and retain their normal plane eligibility.

Root-surface and border hit testing use the same logical corner geometry. Popup
hit testing and committed resize input coordinates retain their existing rules.

## Verification

- Corner geometry and commit-cache unit tests cover cutouts, small windows, and
  stable versus changed content.
- Ignored headless EGL tests compare GPU pixels with a CPU reference for all eight
  output transforms, scales 1, 1.5, and 2, and partly offscreen windows.
- A real-window EGL test checks root input, element identity, and zero idle damage.
- A resize-blend EGL test checks selected blend colors, corner opacity metadata,
  and the square fullscreen path.

Run GPU checks with `cargo test --locked -p raven --lib rounded -- --ignored`
and `cargo test --locked -p raven --lib blended_resize_clips -- --ignored`.
These tests do not replace live DMA-BUF, KMS, dragging, and fullscreen validation.
