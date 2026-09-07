# Input timing

Set RAVEN_INPUT_TIMING=1 to enable batched compositor-boundary observations. The default path reads no diagnostic clocks and allocates no timing state. Combine with RAVEN_FRAME_TIMING=1 when measuring a real VT session and redirect stderr to a file rather than rendering the logs in a terminal.

The collector records source timestamp age at dispatch, observation age at explicit flush attempts and accepted frame queues, frame timing intervals and conditional GPU sync waits. Coalesced observations retain oldest and latest ages. Reports run every two seconds through the existing deadline source. Storage is constant: at most one pending KMS snapshot and one software successor. Reporting retains both; reset on VT pause and successful resume drops both, all input batches and partial reports.

## Hook contract

- Call `frame_queued(false)` once after successful actual KMS submission with no pending frame.
- Call `frame_queued(true)` once after software accepts a successor behind exactly one pending KMS frame. Neither an empty render nor a failed queue gets a snapshot. Do not call again when that successor reaches KMS.
- Call `presented(metadata, successor_submitted)` for the selected CRTC after the scheduler accepts the pending flip and after `compositor.frame_submitted` returns. Pass the original DRM metadata, not a synthesized fallback. The boolean is true only if that call succeeded and a software successor existed.

Presentation metadata and observation ages always belong to the old pending snapshot. A true boolean promotes the successor and stamps its KMS submission boundary with the current hook clock. Its original acceptance timestamp is unchanged. A false boolean discards any successor as a failed deferred submission; the old frame still gets its presentation. Missing or invalid metadata does not prevent promotion or discard. No presentation is fabricated for a discarded frame.

## Report fields

All sample fields use `n:min/mean/max` in milliseconds. Hook clocks use CLOCK_MONOTONIC. They are post-operation observations, not exact ioctl entry or return timestamps.

- `accepted_queue_hook_to_drm_ms` measures successful acceptance hook to kernel presentation. It includes software residence for deferred frames and replaces `queue_return_to_drm_ms`.
- `software_accept_hook_to_kms_submit_hook_ms` measures deferred acceptance hook to the presentation hook that observes successful successor submission. Only promoted successors contribute; direct submissions and failed successors do not.
- `kms_submit_hook_to_drm_ms` measures actual submission hook to kernel presentation. Direct frames use their queue hook clock; promoted frames use the preceding presentation hook clock. Missing clocks do not fall back to acceptance time.
- `drm_to_presented_hook_ms` replaces `drm_dispatch_ms`. It measures kernel presentation to this hook, including dispatch delay and work in `compositor.frame_submitted`, potentially including successor submission. It is not callback-entry latency.
- `queue_*` observation ages end at accepted queue time. `presentation_snapshot_*` ages use the old pending frame's observations and kernel timestamp. Batch and observation counts remain available when timing samples cannot be recorded.
- `queued` counts all acceptance hooks; `deferred_queued` counts those marked deferred. `successor_promoted` counts tracked promotions and `successor_submit_failed` counts tracked successors discarded on a false boolean. `pending`, `successor` and `pending_ambiguous` describe state at report time.
- `queue_invariant_mismatch` replaces `queue_overlap`. Orphan deferred acceptance, overlapping direct submission or a third snapshot invalidates attribution. `snapshots_discarded` counts all invalidated snapshots, including the new acceptance, plus failed successors. Reset discards partial reports rather than counting drops across sessions.
- `successor_invariant_mismatch` counts a true submission boolean without a tracked successor. The old snapshot remains attributable, but the unknown next frame is marked ambiguous. An ambiguous pipeline stays unattributable until a presentation with a false boolean drains it, or reset clears it. `presentation_ambiguous` and `presentation_without_snapshot` count presentations lacking attribution.
- Missing, realtime, zero, future and regressed DRM timestamps and unavailable clocks have separate counters. `drm_before_queue` and `drm_before_kms_submission` reject presentation ages and frame intervals before their known boundaries. `phase_negative` also counts a negative software residence interval.

These are not input-to-photon or causal browser/application-response measurements. A cursor-only frame may contain the latest pointer position while an application still renders. Flush completion does not acknowledge client receipt. Observation snapshots make no claim about the contents of client buffers.
