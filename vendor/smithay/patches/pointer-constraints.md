# Pointer constraints on Smithay 0.7

This patch is original Raven code. The behavior was researched through
prefromance-inputlag/research/game-input-dialogs/pointer.md, which identifies
niri revision dd75865f547f0eac0e9b6c4d86d2cd00c0744252. No niri source was copied.
It does not change dependency versions or the existing viewport patches.

## Changed APIs

- PointerHandle::set_motion_hook installs an optional MotionHook function.
  MotionPhase::Before receives old and proposed grab-selected focus and the
  motion event. Returning false suppresses delivery and leaves the pointer's
  actual focus and location unchanged. MotionPhase::After observes delivery.
  The hook runs under the pointer mutex. It must not query or dispatch through
  methods that lock that pointer. Raven uses the supplied targets and cached
  focus instead. Compositors that do not install a hook retain existing behavior.
- PointerConstraintsHandler::constraint_committed is a default no-op callback
  after committed regions and hints have been applied.
- PointerConstraintsHandler::constraint_destroyed is a default no-op callback
  receiving the removed PointerConstraint by value. It runs after the surface
  and constraint mutexes have been released. Raven may restore a committed hint
  here, but never from the motion hook or a forced release.
- Locked and confined resource dispatch now requires PointerConstraintsHandler.
  Existing new_constraint and cursor_position_hint signatures are unchanged.
- PointerConstraintRef::activate and deactivate ignore redundant calls.
  Inactive one-shot constraints are not consumed by a redundant deactivate.
  Deactivation still retires an active one-shot entry and retains a persistent
  entry. The old claim that focus loss automatically deactivates constraints
  has been corrected. Raven installs the motion hook to provide that lifecycle.

## Internal fixes

Constraint destruction compares protocol object identity before removing an
entry. Requests on retired one-shot resources also check identity, so an old
resource cannot change or remove its replacement.

The nested committed helper snapshots regions and hints in a pre-commit hook
using Smithay's surface cache. Post-commit application follows the surface
transaction, including synchronized subsurfaces and delayed commits. Later
uncommitted requests cannot overwrite an earlier transaction's state. Cached
updates are matched by protocol object identity. Hint callbacks and commit
callbacks run outside both surface and constraint locks.

## Validation

The one Raven regression is
input::pointer::capture::tests::real_wire_game_capture_delivery_and_lifecycle.
It binds both globals on a Unix socket and exercises actual Wayland resources,
event payloads, grabs, and constraint lifetimes. It shares the existing desktop
wire transport and toplevel fixture without modifying them.

Run with cargo test -p raven --offline --locked --lib
input::pointer::capture::tests::real_wire_game_capture_delivery_and_lifecycle
-- --exact.

No live compositor, game, satellite, or session restart was used. The regression
does not measure device sensitivity or latency, or separately exercise delayed
surface transactions and ancestor destruction. Those paths were inspected in
Smithay's cache and destruction implementations.
