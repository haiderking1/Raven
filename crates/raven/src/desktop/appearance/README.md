# Appearance

Appearance uses logical pixels and has no Lua or animation dependency. Defaults
are 8 pixels between columns and stack rows, 8 pixels on each workarea edge,
and a restrained 2-pixel border. Each gap can be set independently. A zero gap
removes that spacing; a zero border width removes the frame. Appearance::disabled()
sets every gap and the border width to zero.

State::appearance returns the current settings. State::set_appearance validates
all fields before changing state or sending configures. Invalid settings preserve
the old settings, geometry, focus and redraw state. Applying identical settings
is a no-op. Extents must be in 0..=65535. Both border colors use straight RGBA;
each channel must be finite and in 0..=1. The renderer premultiplies RGB by alpha.

## Allocation contracts

- State::window_frame_geometry returns the allocated rectangle including borders.
- State::window_client_geometry returns the interior allocation used for client
  configures, root-surface rendering clips and input clips.
- window_layout_geometry remains an internal alias for the client allocation.
- floating_geometry retains the normal client allocation during fullscreen.
- Space locations refer to client geometry origins. window_surface_origin also
  subtracts the client's XDG geometry offset to locate its buffer tree.

Popups may extend beyond the client allocation but remain clipped to the output.
Border hits focus their containing window without delivering pointer events to
an unrelated wl_surface. Client input regions still apply inside the allocation.
Oversized client buffers cannot receive input or render content in gaps.

## Workareas and small outputs

Layer reservations apply before outer gaps. Opposing outer gaps shrink
proportionally if they would consume the workarea. Inner gaps shrink before
partitioning columns and rows. Integer remainders stay with the right column and
its first rows. Tile order is independent of focus and stacking order.

If even one pixel per tile cannot fit, tiles overlap with positive allocations.
A fully reserved workarea uses one pixel inside the output. Border width is
limited by both frame dimensions so the client keeps at least one pixel.

Floating placement centers and clamps the entire frame within the gapped
workarea. Natural client size is separate from the clamped allocation. Commits
that obey a clamp, including delayed acknowledgments after bounds relax, do not
erase that size. An unconstrained opening floating axis still offers client size
choice until its first buffer, except when the bounds allow only one pixel.

The displayed fullscreen owner uses its existing full-output allocation with no
gap or border, including the existing commit-gated exit. Related floating dialogs
keep their normal frames and workarea bounds. Fullscreen scheduling is unchanged.

## Rendering and locks

Mapped tile, floating and transient frames are cached during layout/configure
changes. Border rendering does not arrange windows. Each Window owns four stable
SolidColorBuffers and cached immutable solid elements. Scene rebuilds share those
elements without copying their opaque-region allocations. Geometry or color
changes update the affected snapshots; unchanged snapshots retain their damage
identity and commit counter. The cache holds no client buffer or texture refs.

Physical shared edges are rounded before stripes are built. Corners belong only
to the top and bottom stripes, so translucent borders do not double-blend. Existing
redraw requests and damage tracking handle changes; borders add no timer or poll.
Layer render passes release their layer_map guard before any window lookup.

The single appearance wire regression covers default and changed configures,
invalid/no-op application, disabled layout, input clipping, layer workareas,
floating-size restoration, fullscreen preservation and tiny allocations. The
existing ignored EGL render-lock regression retains its original assertion first,
then draws borders and checks stable IDs, idle damage and frame removal.
