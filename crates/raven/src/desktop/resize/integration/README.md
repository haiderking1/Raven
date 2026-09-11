# Resize integration notes

## Delayed configure acknowledgement state

Smithay 0.7's XDG post-commit hook reads the latest acknowledged state when a
queued surface commit applies. Adding resize blockers exposed an ordering risk:
an older commit could inherit a newer acknowledgement and change fullscreen
ownership using a state that was not acknowledged when that commit was submitted.

resize/role.rs records the acknowledged serial and state in a Cacheable value
at pre-commit. Smithay carries it with the surface transaction. Raven restores
that applied value at the start of CompositorHandler::commit, before renderer,
window geometry, fullscreen and animation consumers. No vendor edits or version
changes were needed. This is a source-identified integration issue, not a claim
that it caused the user's original browser flicker.

## Integration build

Formatting, locked/offline production library and binary checking, the release
build and git diff whitespace checking passed. The production build has no Raven
warnings; the eight existing vendored Smithay warnings remain.

The initial compiler pass found a missing direct tracing dependency reference,
a snapshot Debug derive involving non-Debug scene elements, and an overlapping
fullscreen borrow. Logging now uses Raven's existing stderr convention, snapshot
diagnostics report retained input count, and the decision is computed before
mutable fullscreen visibility lookup. A renderer helper's visibility was narrowed
to match its private scene-element API. No dependency was added.

No tests were added or run, per user request. No compositor, browser or isolated
GPU validation was launched. The build does not establish live animation quality,
absence of browser flicker, GPU shader execution or input-latency performance.
The frame scheduler, GPU timing implementation, submission module, dependency
pins and vendored sources were left unchanged. Nothing was committed or pushed.

See the animation renderer documentation for GPU measurement-scope limitations.
Detailed command results and the preserved pre-change release binary remain in
ignored prefromance-inputlag artifacts.
