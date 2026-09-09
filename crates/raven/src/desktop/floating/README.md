# Automatic floating

Raven opens a toplevel as floating when it has an xdg-toplevel parent, or when
its committed minimum height is positive and equals its maximum height.
Fixed width alone does not qualify. There are no title, app-id, Steam or other
application-specific rules. These are the two default mechanisms described in
prefromance-inputlag/research/game-input-dialogs/dialogs.md. This implementation
is original Raven code; no niri source was copied.

## Mapping and ownership

The first configure follows the bufferless surface commit. Size hints come from
Smithay's committed XDG cached state, not pending requests. Parent changes are
immediate in XDG. Relevant hints can change the opening classification until the
first buffer maps; ordinary mapped commits only update size, not layout mode.
Null-buffer unmap ends that mapping cycle and permits fresh classification.

A valid parent chain supplies the opening workspace without switching to it or
focusing a hidden dialog. Mapped reparenting does not move workspaces or change
layout mode. It repairs stacking, placement and fullscreen visibility. Cyclic
parent chains are ignored for inheritance, centering and ancestry ordering;
a parent hint still qualifies the window for floating. Destruction leaves mapped
orphans floating, centered on the workarea when no mapped parent remains.

Each workspace owns floating placement separately from its tile order. Floaters
never occupy a tile, including while fullscreen. The upper stack also admits a
mapped tiled window reparented under another window: it retains its tile
allocation but is raised above its ancestors. Stable ancestry ordering preserves
sibling order across layout changes. Focus can raise a branch, never put a parent
above its descendants.

## Size, placement and fullscreen

Unconstrained initial axes use XDG's zero-size client choice; fixed axes receive
their bounded fixed size. Workarea bounds are advertised before mapping. Once a
buffer arrives, its window geometry supplies the natural size. Min/max hints and
the current workarea bound that size; the workarea wins if the client's minimum
cannot fit. A client complying with a workarea clamp does not erase its larger
preferred size. Client-side window geometry offsets remain part of surface-origin
calculation, not the stored floating location.

A dialog is centered over its same-workspace mapped parent's layout allocation,
then clamped to the workarea. Other floaters use workarea centering. Placement is
recomputed on client resize, ancestor placement/reparenting, workspace movement,
workarea/output changes and destruction. Raven still has one live output.

Fullscreen keeps the floating entry and preferred size. Its existing requested,
committed and displayed ownership states still govern transitions and grabs.
Exit targets the bounded floating allocation, with tiled flags cleared, rather
than inserting a tile. Unacknowledged exits keep their committed fullscreen
allocation; displaced claimants use their normal allocation as before.

Render order, hit testing and visibility use the same cached upper stack.
window_layout_geometry supplies floating clips to existing render/input paths;
popups remain output-clipped. Floating windows remain ordinary mapped Space
members for frame delivery and hidden-workspace suppression. Floating geometry
lookups do not acquire the layer-map lock.

## Modules and accessors

- hints.rs reads committed hints and handles opening classification/inheritance.
- configure.rs applies size constraints and clears tiled/fullscreen flags.
- placement.rs records natural sizes and computes bounded parent-centered layout.
- stacking.rs validates parent chains and repairs stable ancestor ordering.
- lifecycle.rs owns mapping, transfer, cleanup and pre-map bounds refresh.
- access.rs provides read-only State methods for the existing integration suite:
  - window_is_floating(&Window) -> bool includes pre-map/fullscreen floating mode.
  - workspace_floating_count(usize) -> usize counts mapped floating entries on a
    zero-based workspace, including fullscreen entries; invalid indices return 0.
  - floating_geometry(&Window) -> Option<Rectangle<i32, Logical>> returns the
    normal allocation in global logical coordinates, retained while fullscreen.
    It is None before placement or without a usable output/workarea.

No new State/init hook or external runtime export is required for in-crate tests.
There are no general floating movement/resize controls or rule configuration.

## Verification boundary

The existing real-X11 integration regression now creates a transient dialog and
parentless splash through installed satellite. Both float at natural sizes while
the main tile remains unchanged. Scene import, stacking, hit testing, unmapping,
fullscreen and socket cleanup pass. Existing native transient assertions now
expect client-chosen opening sizes and floating allocations rather than tiles.
Only geometry/hint changes arrange floating windows during mapped commits;
ordinary pixel updates do not rebuild the floating stack.

Actual games, physical camera behavior, output hotplug and live workarea changes
still need normal-user session validation. No active session was restarted.
Unknown X11 transient parents that satellite cannot translate remain unsupported.
