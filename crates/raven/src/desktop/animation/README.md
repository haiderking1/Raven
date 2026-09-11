# Snapshot resize animations

Raven captures the old current window body in a compositor-owned GPU image before
a resize-acknowledging surface commit replaces it. Popups are not captured.
Transactions still control whether a commit applies and when displayed allocation
and Space placement change. Animation never supplies transaction correctness.

The default duration is 200 milliseconds with bounded cubic ease-out,
`p = 1 - (1 - t)^3`. Frame and client rectangle edges interpolate independently.
The renderer mixes premultiplied old and current window images by that progress.
This preserves transparency and subsurface overlap without the opacity dip of
fading each surface separately. The current image includes the moving borders.
There is no browser-specific relayout delay or settling heuristic.

## Typed settings

`runtime::settings::ResizeAnimations` is also the type of
`Settings::resize_animations`. `ResizeAnimations::from_millis(u64)` accepts 0
through 2000 milliseconds and rejects larger values. The private duration field
prevents bypassing validation. `ResizeAnimations::OFF` and a zero duration both
bypass capture and animation. A Lua loader can construct the same value without
putting parsing in commit or render paths.

`State::set_resize_animations` cancels existing visuals when the policy changes.
Normal commits and resize transactions continue unchanged when animation is off.

## Commit ordering

1. Central XDG ACK handling records the latest size or fullscreen-state change
   relative to the committed role state. Activation-only ACKs that carry an
   outstanding resize must not erase it.
2. The compositor pre-commit hook consumes that intent and copies OLD current
   content, including root geometry offsets, viewports and subsurfaces. No new
   buffer assignment is consumed or invented.
3. Renderer reconciliation waits for the actual applied role serial and for
   `resize_displayed_frame(window)` to return `None`. Only then is the new
   displayed geometry an animation endpoint.
4. Live current content is rendered every animation frame. An interruption copies
   the last accepted blended image and retains its sampled geometry. It does not
   restart from the previous animation's original endpoint.

The accepted image is the latest successfully queued render, including a reserved
successor. That is the image preceding the next render in Raven's output queue;
an arbitrary wall-clock sample could skip geometry that was never rendered.
While a replacement is blocked, the captured image stays at that accepted pose.
Repeated blocked commits reuse that copy and update its target serial.

## Visibility and input

Only visible mapped windows acquire snapshots. Workspace changes, role destruction,
unmap, output geometry or scale changes, and VT suspension cancel them. ACK intents
also carry the output and workspace context in which they were received.

Popups and layers remain live with their existing stacking, clipping and committed
input coordinates. Toplevel input also remains on committed client geometry,
not on old pixels or a synthetic surface. Niri's pinned tile hit path likewise
consults current client input regions rather than inverse-scaling old snapshot
pixels. Raven does not alter physical relative motion, pointer-lock deltas or the
hidden-workspace input filter.

See the render animation README for fences, resource limits and participation
accounting. This implementation reimplements the mechanism documented at Niri
`dd75865f547f0eac0e9b6c4d86d2cd00c0744252`; it does not transplant GPL source.
