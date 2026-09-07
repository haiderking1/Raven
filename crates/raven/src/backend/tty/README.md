# Experimental display pipeline

The experimental build defaults to `RAVEN_FRAME_PIPELINE=adaptive`. Light compositor workloads retain immediate, single-pending-frame rendering. When estimated render work plus scheduling margin reaches three quarters of a refresh period, it permits one late-prepared successor. It returns to the immediate path below half a period, draining already-accepted work rather than dropping it.

## Ownership and timing

- One KMS-pending frame and at most one software successor. The current scanout buffer is additional.
- Extra preparation happens near the predicted pending pageflip, not immediately on every input event. Commits coalesce until that window opens.
- The lead uses render wall time and asynchronously collected GL elapsed intervals. It is capped at one display period. These are conservative estimates, not an Apple API emulation or a guaranteed deadline.
- Smithay handles native render fences where supported. Its required CPU synchronization fallback remains intact. The renderer is not moved to another thread; driver stalls remain possible.
- Four reusable GPU query slots bound measurement storage. Missing or invalid measurements are not interpreted as zero GPU work.
- Input still flushes before desktop reconciliation and rendering. Ordinary scrolling and physical Ctrl+wheel events are not rewritten.

## Presentation and failure handling

Smithay submits a software successor inside `frame_submitted`. If that submission fails, it can drop userdata for both frames even though the preceding frame completed. Raven retains a separate ticket per accepted snapshot, completes only the old frame with that event, and discards the rejected successor. Shared carriers are explicitly emptied on pause or failure even while Smithay retains its queue.

A recoverable atomic optional-plane rejection schedules one fresh full-composition attempt at end of dispatch. Optional planes remain suspended until VT reactivation. Other submission errors stop the backend. Rejected feedback is never attached to newer content. Preparing a frame cannot restart the watchdog for the older KMS submission.

Callbacks wake clients on successful KMS submission, including successor promotion, not merely on preparation. A same-batch presentation cannot erase an unsent callback cycle.

## Normal-user VT comparison

```sh
RAVEN_FRAME_PIPELINE=adaptive RAVEN_FRAME_TIMING=1 RAVEN_INPUT_TIMING=1 cargo run --release --locked -p raven 2>/tmp/raven-adaptive.log
```

Use `RAVEN_FRAME_PIPELINE=deadline` to exercise late overlap even under light load. Use `RAVEN_FRAME_PIPELINE=immediate` to disable render-ahead and GPU queries for a compatibility comparison. Invalid values fail startup. Keep the browser, page, zoom operation, display mode and release profile the same. Do not launch another DRM session over a running desktop to automate this test.

Compare scrolling and cursor responsiveness as well as Ctrl+wheel zoom, workspace changes and VT return. Output flip counts do not establish browser content FPS or input-to-photon latency. Page layout and browser raster production remain client work. There is no desktop-magnification substitute, forced VRR, driver tuning, refresh-mode change or dependency upgrade in this experiment.

## Diagnostics

Existing `*_queued` plane counters count accepted compositor snapshots, including software successors. `software_queued` separates those successors; `kms_submitted` counts actual successful KMS handoffs. Neither is a presentation count. `gpu_batch_max_ms` reports the largest valid GL interval per nonblocking collection batch, not a per-frame average or isolated shader cost. Input reports distinguish accepted-queue residence from actual-KMS-submission-to-presentation.

See [scheduling](schedule/README.md), [GPU queries](gpu_time/README.md), [presentation](presentation/README.md), and [input timing](../../input/timing/README.md).

Documented design references, not proprietary code: [Apple render loop](https://developer.apple.com/videos/play/tech-talks/10855/), [Metal display link](https://developer.apple.com/documentation/quartzcore/cametaldisplaylink), and [drawable synchronization](https://developer.apple.com/library/archive/documentation/3DDrawing/Conceptual/MTLBestPracticesGuide/Drawables.html).
