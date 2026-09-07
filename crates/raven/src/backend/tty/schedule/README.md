# Frame scheduling

Repaints come from applied client commits and desktop/input changes. Requests coalesce while one DRM frame is pending. A pageflip releases that slot; it does not itself request rendering.

After a successful queue, client buffers are latched. Raven advances the callback cycle and wakes visible clients immediately, before the pageflip, so their next buffers can become ready while KMS presents the queued frame. End-of-dispatch rendering runs before callback delivery when both are ready.

Each surface receives callbacks at most once per cycle, including surfaces reached through more than one tree. Pageflip and empty-render callback passes revisit the current cycle rather than advancing it. This prevents callback-only commits from creating a busy loop.

A no-damage render arms one callback wake on the refresh grid. Accepted monotonic DRM timestamps anchor that grid; unknown timestamps use dispatch time. Late timer delivery does not move the grid. A successful queue cancels the estimated wake. After an estimated wake, the backend stays idle unless more work arrives.

The three-second pending-frame watchdog remains separate. Pause clears scheduling work; resume resets the clock, discards stale compositor frames, and requests a fresh render. No buffer is reused merely because a timer expired.
