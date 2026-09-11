# Coordinated resize transactions

Raven coordinates visible Wayland windows in one active-workspace resize batch.
This subsystem does not schedule frames or animate them.

## Surface and layout application

Layout code brackets a change with begin_resize_batch and end_resize_batch.
Nested tiling, floating and fullscreen operations share the outer batch. Collection
captures each eligible mapped window's displayed frame, client rectangle and Space
location before layout changes its targets. Hidden windows, unmapped clients and
windows without a Wayland toplevel do not become participants.

Layout still computes target tiles, floating allocations and fullscreen configures.
place_resize_window records target locations while keeping Space at the displayed
location. window_frame_geometry and window_client_geometry expose the retained
allocation. Configure code uses target tiles, floating geometry or the explicit
fullscreen target. Floating parent placement uses window_target_client_geometry,
not the retained displayed rectangle.

The surface pre-commit hook adds a Smithay commit blocker for resizing participants.
Position-only siblings keep updating their content at the old displayed location.
Even an older response
cannot replace current surface state while its cohort waits. A matching response
must acknowledge the expected serial or a newer serial carrying the issued or
latest target size and fullscreen state. Serial comparison uses Smithay's
wrap-aware Serial ordering.

A coalesced ping releases a ready cohort, including a single ready participant.
There is no minimum wait. Release opens the coordination gate and calls each
client's compositor_state.blocker_cleared. Displayed allocations remain retained
through all resulting surface callbacks. Only after those callbacks return does
Raven publish matching applied allocations and locations, preserving stack order.
Fullscreen's existing committed ownership and grab checks still decide its mode
and location. Reentrant commit callbacks defer tiling membership and allocation
recomputation through a fixed workspace bitmap. After publication, those changes
start a follow-up batch from the now-displayed state. They cannot publish a newly
requested size while the previous cohort is still applying.

## Deadline and acquire safety

The batch has one one-shot calloop timer, due 300 ms after collection starts.
Superseding requests keep that deadline. No-op layout refreshes neither restart
timed-out cohorts nor probe client commit queues. Readiness arrives through pre-commit,
DMA-BUF fd callbacks and destruction events, not an idle poll or frame tick.

The coordination blocker and the DMA-BUF acquire blocker are independent.
Per-surface acquire counters include earlier dependent attachments, and root
readiness includes synchronized descendants. A transaction timeout never changes
an acquire fence or its cancellation state.

At the deadline, responsive clients can proceed without missing clients. A window
whose matching state has not applied retains its displayed allocation until that
state actually applies. It no longer blocks peers. This deliberately allows partial
progress after timeout; it cannot promise atomic completion from an unresponsive
client. Destruction, unmap, workspace moves/switches, output invalidation and VT
suspension remove obsolete holds and wake coordination without bypassing acquires.

## Bounded state and supersession

Raven stores one held allocation and latest configure expectation per window, one
cohort and one deadline. Size-only changes retain one outstanding request and
replace Smithay's server_pending target. The next matching response sends that
latest target and stays gated until it responds. Intermediate input sizes do not
create Raven transaction queues. Fullscreen's required explicit configure replies
can supersede size requests and retain their existing ownership serial path.

A newer activation or decoration configure can carry a resize target. The commit
hook recognizes that serial too, including when another protocol path flushed
server_pending. GPU readiness storage is bounded by surface count, not input count.
Smithay still owns its normal protocol configure and committed-surface queues.

## Pinned Smithay role state

Raven's Smithay 0.7 copies last_acked into role.current at application time. That
would let an old blocked commit borrow a later ACK. role.rs implements Cacheable
for the complete acknowledged ToplevelState and serial. Its pre-commit capture
travels with Smithay's surface cache. CompositorHandler::commit restores this
applied state before existing fullscreen, floating and animation consumers run.
No vendor change or separate ACK queue is needed.

## Hooks

- Install State::install_resize_transactions once before client dispatch.
- Install the surface hook after DMA-BUF acquire setup, before animation capture.
- Restore apply_role_state first in CompositorHandler::commit.
- Call resize_surface_applied after commit_window.
- Send tiling/floating pending configures through send_resize_configure.
- Track explicit fullscreen serials with track_resize_configure.
- Route layout positions through place_resize_window.

Animation uses the displayed allocation and actual applied surface state. It does
not supply configure sizes, and transaction release does not require an animation
API. Source files divide batch collection, configure coalescing, readiness, role
caching, surface hooks, geometry publication, release, runtime and lifecycle work.
