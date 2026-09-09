# Fullscreen

Raven supports xdg-toplevel fullscreen and Super+F. Maximize remains unadvertised
and requests leave the current mode unchanged. There is one displayed fullscreen
owner per workspace; optional output hints fall back to the sole live output.

## Requests and committed content

- Bufferless startup requests record intent. The first commit sends one initial
  configure with the final requested mode, size and tiling edges.
- A mapped unfocused client cannot claim fullscreen. The current claimant,
  remembered workspace focus or actual keyboard owner may request it. Unmapped
  startup claims take effect on mapping, not on the request.
- Requested ownership, the displayed owner, applied geometry and outstanding
  configure transitions are separate. ACK alone never changes the scene.
- Smithay applies the acknowledged state before Raven handles the surface commit.
  Raven requires the latest transition serial or a newer configure carrying the
  same mode and size. Serial comparisons use Smithay’s wrap-aware ordering.
- A replacement configures the displaced claimant out of fullscreen. Old commits
  cannot reclaim ownership. Once displaced, its buffer is confined to its normal
  layout allocation even if its exit response arrives after the replacement exits.
- A transition that would hide a held-button or non-popup keyboard grab waits
  until release. Popup grabs belonging to trees being hidden are dismissed.

## Geometry, visibility and focus

Fullscreen uses the full logical output rectangle, including its origin, rather
than the layer-reserved workarea. Smaller client window geometry is centered over
opaque black. Surface origins include the client’s window-geometry offset.
Toplevel/subsurface rendering and input are clipped to their allocation; popup
trees can escape a tile but not the output. Reactive popups are reconstrained when
parent placement changes, without repeatedly configuring unchanged geometry.

Related mapped transient toplevel dialogs stay centered above the owner. Parent
chains are bounded against cycles, and descendants remain above parents when
focus raises their parent. New unmapped dialogs inherit their parent’s workspace
without switching the active workspace. Opening dialogs use the separate
[floating layout](../floating/README.md); ordinary tile membership/order survives.

Overlay layers remain visible. Top layers remain above fullscreen only when they
request exclusive keyboard interaction, allowing launchers without app-specific
rules. Other Top layers, Bottom/Background layers and unrelated windows are
suppressed consistently in rendering, hit testing, focus and popup-grab admission.
Mapped suppressed clients keep throttled occluded callbacks; inactive workspaces
remain excluded. Presentation and scanout still use accepted render visibility.

## Lifecycle

Exit restores the current tile for tiled windows, or the preserved natural size
bounded to the current workarea for floating windows. Floating mode survives
fullscreen without gaining tile membership. Workspace moves preserve
intent/displayed ownership without following; a
destination claimant is displaced explicitly. Null-buffer unmap, destruction and
disconnection clear fullscreen claims, including Smithay’s potentially stale
committed mode on remap. Output loss withdraws intent rather than resurrecting it
when geometry becomes available again.

The implementation remains single-output and native Wayland. These tests do not
establish live DRM presentation correctness, GPU performance or client-specific
fullscreen behavior; the release still needs normal-user session testing.

Focused wire regressions live in `../tests/fullscreen/`. Normal tiled rendering
keeps borrowed stack iteration, and clipping uses direct tile lookup rather than
allocating all tiles for each window. No browser-specific scheduling was added.
