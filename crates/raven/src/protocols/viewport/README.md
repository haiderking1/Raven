# Viewporter

Raven advertises `wp_viewporter` version 1 with Smithay 0.7 public APIs.
`State::new` creates `ViewporterState` and retains it in `_viewporter_state`.
`delegate_viewporter!(State)` dispatches both the global and each `wp_viewport`.
Smithay needs no Raven handler trait or state getter for this protocol.

## State and commit integration

Smithay owns the per-surface `ViewportCachedState`. `set_source` writes the
pending logical source rectangle after buffer transform and buffer scale.
`set_destination` writes the pending logical destination size. Neither request
changes the displayed surface before its surface state is applied. A source
with all four values set to `-1` unsets cropping. A destination of `-1, -1`
unsets scaling. Other invalid nonpositive sizes and negative source origins
raise `bad_value`.

The compositor transaction machinery promotes this cached state on commit,
including synchronized subsurface state when the parent transaction applies.
Raven already calls `on_commit_buffer_handler::<Self>` first in
`protocols/compositor/mod.rs`, before `popup_manager.commit`, the synchronized
subsurface early return, and `commit_window` or `commit_layer`.
`desktop/lifecycle.rs` calls `window.on_commit` after that buffer handler.
This ordering is correct. No desktop or compositor ordering edit is required.

The buffer handler reads the current viewport, checks the source against the
transformed and scaled buffer bounds, and computes Smithay's surface view.
An out-of-bounds source raises `out_of_buffer`. A destination supplies the
surface size; without one, the source size supplies it; with neither, the
buffer's logical size supplies it. Viewport-only commits update the view of an
already attached buffer without requiring another attachment. A surface with
no buffer remains unmapped even if it has a destination.

The existing `render_elements_from_surface_tree` path uses the source for
texture sampling and the destination for geometry. Smithay's surface-tree
bounds and `Window::surface_under` or layer `surface_under` use that same
destination and the surface input region. Pointer coordinates stay in surface
logical coordinates, not cropped buffer coordinates. Raven's tile and output
clipping still apply. No second transform, input-coordinate rescaling, or
manual mutation of `RendererSurfaceState` belongs in this module.

## Teardown

- Destroying `wp_viewporter` leaves its existing `wp_viewport` objects intact.
- Destroying a `wp_viewport` clears its surface association immediately and
  resets both pending viewport fields. The old current crop and destination
  remain until the next applicable surface commit. A synchronized child still
  waits for its parent transaction. A replacement viewport may be created
  before that commit; its requests update the same pending cached state.
- Attaching a null buffer unmaps the surface and resets Smithay's renderer
  resources when the commit applies. It does not destroy the viewport or unset
  its protocol state. A later buffer attachment uses the retained viewport.
- Destroying `wl_surface` ends its surface state and renderer resources.
  Smithay stores a weak surface reference in `wp_viewport`; subsequent valid
  setter requests raise `no_surface`, while `destroy` remains legal.
- The global is display-wide, not tied to a DRM output, renderer, or session
  activation. Backend teardown must not remove it or clear surface viewport
  state. Existing display teardown owns its final lifetime.

Smithay 0.7's `ViewporterState` contains a `GlobalId` but has no `Drop`
implementation. Dropping this value alone does not unregister the global,
despite its constructor documentation. If Raven later needs to withdraw it
while retaining the display, use `DisplayHandle::disable_global::<State>`
and eventual `remove_global::<State>` with `ViewporterState::global()`.
Neither operation destroys already bound protocol objects. Keep delegation
available while clients can still send requests on those objects. This change
does not introduce a global-withdrawal lifecycle.

## Smithay patch follow-up outside this directory

The inspected registry source for pinned Smithay 0.7.0 has a validation-hook
installation bug in `src/wayland/viewporter/mod.rs`.
`ensure_viewport_valid` inserts `ViewporterSurfaceState` even when the surface
has no viewport. This happens during an ordinary buffered commit. Later,
`GetViewport` installs `viewport_pre_commit_hook` only if inserting that same
map entry succeeds. A first viewport created after such a commit therefore
misses the hook that rejects fractional source dimensions without a
destination via `bad_size`.

The authorized Smithay patch needs to track hook installation independently
of map-entry existence. Install it exactly once per surface, including when
a renderer validation call created the entry first, and retain it across
viewport destruction and recreation. Do not work around this by changing
Raven's renderer cache or installing a second Raven viewport state machine.
Dependency and patch files are outside this worker's ownership and were not
edited here. This integration depends on that follow-up for the affected
validation case.

No fractional-scale protocol or satellite changes belong to this delegation
module. No tests were added or run by this worker; the main agent owns the
single regression.
