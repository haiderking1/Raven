# Game pointer capture

Raven registers zwp_relative_pointer_manager_v1 and zwp_pointer_constraints_v1.
Capture uses Smithay's actual grab-selected pointer focus, the mapped surface's
input region, and the committed explicit constraint region. Keyboard focus loss,
hidden workspaces, unmap, destruction, and VT suspension force release without
cursor restoration. Persistent constraints can activate again; one-shot
constraints cannot.

Locked physical relative motion takes the relative-motion and frame path only.
It leaves desktop position, pointer focus, and keyboard focus unchanged. Device
accelerated and unaccelerated deltas and the microsecond timestamp are passed
through unchanged. Absolute motion cannot move a locked pointer and does not
invent relative deltas. Buttons, implicit grabs, and the existing scroll path
continue through Smithay.

Confinement clips a motion segment at its first excluded region interval. This
handles ordered added and subtracted rectangles and disconnected regions. It
also rejects endpoints that hit another visible surface. The relative event
still contains the complete device deltas, not the clipped cursor displacement.

Only hints committed during an active lock are eligible for restoration when
the client destroys that lock. Restoration requires unchanged actual focus,
a live mapped surface, a valid surface input position, output containment, and
no pointer grab. Pending hints and hints from forced release are ignored.

## Integration

State::reconcile_pointer_capture validates and activates capture without locking
pointer internals. State::release_pointer_capture forces release without a warp.
State::captured_pointer_location applies lock or confinement to proposed motion.
State::pointer_is_captured and pointer_is_locked guard desktop focus and refresh.
State::pointer_capture_keyboard_focus records keyboard focus.
State::pointer_capture_hint records active committed hints.
State::pointer_capture_destroyed handles protocol removal and optional restoration.
State::pointer_capture_surface_commit and pointer_capture_surface_gone handle
surface and ancestor lifecycle. State::suspend_pointer_capture and
resume_pointer_capture follow libinput suspension and resumption.

The vendored PointerHandle motion hook sees grab clear and restore operations as
well as ordinary dispatch. It is the final guard against synthetic locked motion.
It records actual focus and position without querying the locked pointer handle.

Scene changes must keep calling State::refresh_tiling_pointer or
State::refresh_pointer_focus after mapping, stacking, visibility, or origin
changes. The existing desktop paths provide those calls. Future floating paths
need the same hook; no other desktop file was changed for capture. VT resume adds
one input-lifecycle call in backend/tty/events.rs.

Vendor changes and exact Smithay APIs are documented in
vendor/smithay/patches/pointer-constraints.md. The single real-wire regression
covers delivery and lifecycle, not physical device behavior or game sensitivity.
