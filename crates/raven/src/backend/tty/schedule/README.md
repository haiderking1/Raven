# Experimental display-deadline scheduling

This scheduler owns opaque per-frame tickets. It knows nothing about Wayland, buffers, fences, or KMS objects. There is one KMS-pending slot and at most one rendered successor. There is no growing backlog or replacement of an occupied slot.

## Policy and API

- `Schedule::<T>::new(refresh_millihz, now)` is a test-only compatibility constructor selecting `Immediate`. It renders only when the KMS slot is empty, as the old scheduler did.
- `Schedule::with_policy(refresh_millihz, now, policy)` selects explicitly. `Policy::from_env() -> Result<Policy, PolicyError>` reads `RAVEN_FRAME_PIPELINE`. An unset variable selects `Adaptive`. Only the exact values `adaptive`, `immediate` and `deadline` are accepted. Empty, misspelled, and non-Unicode values are errors. Constructors do not read the environment.
- `render_due(now: Instant)` answers whether requested work can render now. Startup and resume request one render. `request_redraw()` coalesces later requests without capturing their content. The caller must use the latest committed state when the window opens.
- `rendered(ticket: Option<T>, now)` consumes a render result. `None` means no damage. With no pending ticket, `Some(ticket)` records an accepted initial submission. Otherwise it owns the deferred successor. Check `render_due` before rendering and record the result before reporting timing samples. An out-of-window or over-capacity call drops its argument without replacing tickets or consuming the outstanding redraw request.
- `presented(kernel_time, successor_submit_time) -> Option<T>` returns the completed ticket and promotes the successor. The two times have different jobs. Kernel time anchors display prediction; the actual successor submission time starts its watchdog. Unexpected pageflips return `None` without changing the clock.
- `pending() -> Option<&T>` and `queued() -> Option<&T>` borrow the owned slots. Smithay submits the successor inside `frame_submitted`; record the result and actual handoff time. Never submit or render that ticket a second time.
- `discard_pending() -> Option<T>` is only for an immediate failure of that deferred submission, before any further rendering. It returns the failed ticket and requests a fresh render. It must never release an accepted KMS submission or recover from a watchdog timeout.

`active`, `request_redraw`, `callbacks_due`, `callbacks_advance_cycle`, `callbacks_sent`, `deadline`, `stalled`, `pause`, and `resume` retain their roles. All event times use the same monotonic `Instant` domain. Unknown kernel timestamps use dispatch time without claiming hardware-clock accuracy.

## Bounded overlap

`Adaptive` starts in immediate mode. Estimated lead at or above three quarters of a period enables deadline overlap; lead at or below half a period disables it. Hysteresis avoids repeated policy changes from small sample variations. Already-queued work drains normally on the transition back. `Deadline` forces the overlap window for comparison. These thresholds are Raven heuristics, not Apple guarantees.

Under `Deadline`, an empty pipeline renders requested work immediately. With a pending ticket and no successor, rendering waits until:

`clock.next(pending_submission_time) - adaptive_lead`

The prediction stays tied to the pending submission, not the current dispatch time. A missed window therefore stays open rather than chasing future refreshes. A queued successor blocks all further rendering until promotion, even when more commits arrive. Those commits remain one coalesced redraw request.

`deadline()` returns the earliest relevant render window, no-damage callback wake, or pending watchdog. It does not arm a render window without a redraw request. An empty pipeline without work has no timer. A full pipeline has no repeated render wake. The caller must process due work and then rearm from the new deadline, rather than repeatedly rearming an already-due timer without handling it.

The three-second watchdog always belongs to the oldest KMS-pending ticket. Rendering a successor or an empty frame cannot restart it. Promotion gives the successor a fresh timestamp, even when delivery of the old kernel event was delayed. A timeout never frees either ticket automatically.

## Adaptive lead

`observe_render(cpu: Duration)` accepts render wall time spent constructing and submitting rendering, including required driver or synchronization waits. `observe_gpu(gpu: Duration)` accepts a GL elapsed interval from an already-available timer query. Dependency stalls and command-stream gaps can be included. Poll availability elsewhere and skip unavailable results. Neither API waits or reads hardware.

CPU submission-to-fence latency is not a GPU elapsed sample. It includes queueing and CPU/GPU overlap and would count that delay again. CPU wall time and the GL interval may overlap, especially when fallback synchronization was required. This scheduler deliberately sums the separate estimates rather than taking their maximum or pretending their sum measures the actual critical path. That conservative choice can start rendering earlier than necessary. Integration must keep query results associated with their actual render workload and discard stale results across pause/resume.

Each estimate starts at 1 ms. A larger sample raises it immediately. A smaller sample closes 1/16 of the gap per observation. Each sample is capped at one refresh period before updating, so a pathological observation cannot poison the estimate indefinitely or overflow arithmetic. Missing GPU samples retain the previous estimate, including the startup estimate; they do not imply zero GPU work.

The lead is the CPU estimate plus the GPU estimate plus a fixed 500 microsecond scheduling margin, capped at one display period. The small margin covers ordinary dispatch and submission jitter; it is not a guarantee against OS scheduling delays. There is no adaptive wakeup-lateness term. Timing collection and timer delivery belong to backend integration.

## Callbacks and lifecycle

Accepted initial submission advances the callback cycle immediately, before pageflip. Merely preparing a successor does not advance callbacks. Its promotion to submission does. Main must handle an immediate deferred-submission failure before delivering callbacks and process end-of-dispatch rendering before callback delivery.

Ordinary pageflip and empty-render passes revisit the current cycle. They never downgrade an already-due new cycle, including when submission and pageflip occur in the same event batch. An empty render arms one callback wake on the refresh grid. A late wake does not move that grid. Once sent, the wake clears without idle polling. Per-surface deduplication remains the caller's responsibility.

Pause drops both owned tickets, clears callbacks and requests, and disables deadlines. Resume also drops both tickets, resets the clock and budget, clears callbacks, and requests fresh rendering. Dropping the schedule drops any remaining tickets. Returned completed or failed tickets belong to the caller. Backend teardown must make releasing accepted hardware resources safe before resetting scheduler ownership.

## Files and tests

`state.rs` coordinates scheduling transitions. `flight/` owns tickets and submission timestamps. `budget/` estimates render lead. `policy/` parses policy selection. `clock.rs` maintains the display grid, and `callbacks.rs` tracks callback cycles.

The existing tests use a `Schedule<()>` alias and `Immediate`. Additional focused tests cover the late window, bounded capacity, idle behavior, independent timing estimates, watchdog promotion, ticket destruction, policy errors, and callback ordering. Run the integrated checks with `cargo test --workspace --locked --offline backend::tty::schedule`. See [the backend README](../README.md) for normal-user VT comparison commands.
