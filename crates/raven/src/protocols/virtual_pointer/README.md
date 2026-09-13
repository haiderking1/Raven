# Virtual pointers

Raven advertises wlr-virtual-pointer-unstable-v1 version 2, including version 1
bindings. Clients can inject relative and absolute motion, buttons, and framed
scrolling. Version 2 accepts seat and output hints. Null hints use Raven's
current seat and output. Explicit seat/output mappings retain their identities
and deliver nothing if the mapped seat or output is no longer active.

This is session-wide input access for clients that can connect to Raven's
Wayland socket, without an app-name allowlist or a separate permission prompt.
Only run trusted automation tools. It is not a virtual-keyboard implementation.

## Delivery

Motion and button requests use the same capture, focus, popup authorization,
and drag paths as physical input. A virtual frame flushes the accumulated axis
values, discrete steps converted to value120, source, and axis-stop flags before
sending wl_pointer.frame. Motion and button delivery do not invent extra frames.
Only two axes are retained, rather than an unbounded request queue.

Absolute extents normalize onto logical output geometry, including its origin,
and clamp to the usable bounds. Zero extents are ignored. Relative virtual
motion is already in compositor coordinates and does not apply libinput mouse
acceleration again. Under pointer lock it supplies relative motion without
moving the cursor. Confinement uses the existing pointer capture rules.

Invalid axis and source values report the protocol's specified errors. Unknown
button states and codes outside Linux's BTN/KEY_MAX range are ignored. Discrete
scroll conversion saturates rather than overflowing.

## Ownership and lifetime

Buttons track their physical-device or virtual-resource owners. Only the first
press and final release reach the seat. Duplicate presses and unowned releases
cannot end another source's click or drag. Virtual resource destruction and
client disconnect release only that source's buttons and flush cleanup events.
Destroying the manager does not destroy its existing pointers.

Suspend drains physical releases, clears remaining pointer ownership, and
advances an input epoch. Old scroll frames are discarded across that boundary.
Virtual delivery is disabled while the session is inactive or the backend has
failed. There are no polling timers and no rendering or buffer-ownership changes.

## Tests

cargo test --locked -p raven --test virtual_pointer exercises the production
protocol dispatch with real Wayland requests. It covers versions, output/seat
hints, motion, clicks, framed axes, errors, suspension, manager lifetime,
resource destruction, client loss, and button ownership. End-to-end application
input still needs a running Raven session.
