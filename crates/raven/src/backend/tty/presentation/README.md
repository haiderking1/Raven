# Presentation feedback

The protocol advertises CLOCK_MONOTONIC. After a changed frame is successfully queued, its payload receives feedback from that exact render result before returning to event dispatch. Failed plane attempts do not consume requests or leak their ZeroCopy flags into the composition replacement.

Only surfaces with positive visible area and a non-Skipped render state contribute feedback. Copied cursor BOs do not get ZeroCopy. Actual client primary scanout may receive it.

An accepted pageflip completes the queued payload. Nonzero monotonic kernel timestamps carry HW_CLOCK; unavailable or incompatible timestamps use monotonic dispatch time without it. Refresh comes from the output mode. Counter wraps are extended; a backwards reset reports unknown sequence. Constant zero counters stay zero.

A commit marker follows Smithay cached transactions, including synchronized children and acquire blockers. Applying a superseding commit that requests no feedback discards the older unlatched request. A pre-commit must not discard still-current content before that newer transaction actually applies.

Session reset and shutdown discard queued feedback. No-damage renders do not fabricate a hardware presentation; their requests wait for a real presentation or a subsequent superseding commit/destruction.
