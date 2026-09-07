# Frame timing

Timing is off by default. Set `RAVEN_FRAME_TIMING=1` when starting Raven on a real VT as your normal user. This adds observations only. It does not change mode selection, redraw deadlines, frame callbacks, or the one-frame-in-flight policy.

## Capture

Start with the same debug build used for the reported choppiness:

```sh
RAVEN_FRAME_TIMING=1 cargo run --locked -p raven 2>/tmp/raven-frame-timing.log
```

Move the pointer continuously across the desktop for ten seconds. Then scroll continuously in an application for ten seconds. Leave it idle for a few seconds, then exit with Super+Shift+Q. Keep track of the phase order when sharing results.

```sh
grep 'raven: frame-timing' /tmp/raven-frame-timing.log
```

Reports arrive every two seconds through the deadline timer, independently of rendering. Do not judge an idle desktop by its flip rate. Raven intentionally avoids submitting unchanged frames. Discard the startup interval when comparing steady motion.

For a separate optimized-build comparison, repeat the same actions with `cargo run --release --locked -p raven` and a different log file. Do not mix those results with the debug baseline.

## Fields

All `*_ms` triplets are minimum, mean, and maximum milliseconds. `n/a` means no samples, not zero latency.

- `mode`: configured output refresh rate. At 165 Hz the refresh interval is about 6.06 ms.
- `flips` and `flip_hz`: completed pageflips during the actual reporting span, and their rate. These count submitted display updates, not an individual application's FPS.
- `primary_scanout_queued`, `composition_queued`, `cursor_queued`, `plane_recoveries`: accepted queued-frame plane paths and successful bounded composition recoveries. These are not completed presentations or merely enabled policy flags.
- `draws` and `draw_ms`: render calls that queued a changed frame, including render-element construction, GLES work, required synchronization, and queue submission. Desktop reconciliation runs before this measured section. This is CPU wall time, including any blocking, not a GPU execution-time query.
- `empty` and `empty_ms`: render calls that found no damage and queued nothing, measured separately so they do not dilute the draw average.
- `flip_ms`: spacing between DRM pageflip timestamps. Event-loop dispatch times are not substituted for kernel timestamps.
- `seq_steps`: counts of successive DRM vblank sequence differences of one, two, or at least three. Larger gaps mean more refresh periods between submitted updates. They are not automatically dropped frames, especially while idle.
- `dispatch_ms`: time between the kernel's flip timestamp and Raven handling that event, using the same clock domain.
- `timer_late_ms`: wakeup lateness relative to the armed deadline. Deadlines now cover estimated callback refreshes, the pending-frame watchdog, and enabled timing reports, not periodic render polling.
- `timer_early`: wakeups before the armed deadline. These do not force a render or advance callbacks before their deadline.
- `clock`: the latest DRM timestamp clock. Realtime timestamps can jump if the system clock changes; prefer monotonic captures for pacing analysis.
- `metadata_missing`: missing event metadata. Missing metadata breaks both histories.
- `clock_discontinuities`: non-increasing timestamps or clock-domain changes. These discard only the affected timestamp interval.
- `seq_repeat`, `seq_reset`, and `seq_last`: repeated sequence values, backward sequence jumps, and the last raw counter in this report. A constant counter does not invalidate advancing timestamps. Invalid counter differences are excluded from `seq_steps`, independently of `flip_ms`.

Flip history continues across reports, so an interval crossing a report boundary is retained. Switching away from the VT and back resets all samples and timestamp history; inactive time is excluded.

Recording uses constant-space counters without per-frame allocations or logging. Enabling timing adds clock reads and a batched stderr write every two seconds. Redirect to a file as above rather than rendering the reports in a terminal inside Raven. Omit the environment variable to disable the probes.
