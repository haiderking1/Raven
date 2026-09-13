# Reload and error handling

## Evaluation and publication

The configuration worker reads, evaluates, validates, and prepares a complete
candidate. It never applies settings or launches startup commands. Lua runs in
a fresh state on each evaluation with a 32 MiB Lua allocator limit, a two-million
instruction budget, and a 500 ms elapsed-time check at VM hooks. Files must be
regular files no larger than 1 MiB.

Native directory watches cover ordinary writes, atomic editor renames, and
replacement of the configuration directory. Access-only events are ignored.
Writes are coalesced for 120 ms. A one-second worker-side read comparison recovers
missed notifications and unavailable watches without waking the compositor when
content is unchanged. If the file changes during evaluation, the obsolete result
is discarded.

The main loop receives only prepared candidates or diagnostics. It applies a
valid candidate at a dispatch boundary after existing held/live resize cohorts,
fullscreen transitions, and compositor drags settle. Nothing bypasses configure
acknowledgements, acquire fences, or presentation ownership. The latest received
candidate replaces a pending older one.

No startup process is relaunched, killed, or duplicated by a settings reload.
The only process managed by the configuration service is its own error window.
Programs started through keybindings use the currently applied terminal/launcher
or literal spawn argv.

## Failure

A syntax, runtime, validation, or resource-preparation failure rejects the entire
candidate. The active typed settings remain unchanged. On an invalid initial
configuration, Raven starts with built-in defaults and reports the failure rather
than stranding the user without a compositor. Missing/deleted files on reload
are errors; they are not silently replaced with defaults.

Diagnostics distinguish unreadable Lua syntax, invalid settings, and execution
failures. They show the reason first, repair guidance, and numbered nearby source
lines. Setting errors include field names and received values where available;
unknown settings list valid choices. Syntax locations are labeled as detection
points, not necessarily the original mistake. Runtime locations are labeled as
execution/call sites because Lua tables do not retain per-field source locations.
The full Lua error and callback/runtime traceback follow under Technical details. They go to stderr and an error window
with selectable text and a Copy error button. Escape or Close dismisses the
window. A subsequent valid application closes it automatically. A dismissed
error is not reopened by unchanged periodic file checks; manual reload can show
it again.

The error window runs GTK4 in a separate process. A private inherited Wayland
socket marks only that connection as compositor-owned. This allows the error
window to remain visible above fullscreen without granting privileges based on
a forgeable app ID or title. It uses normal floating geometry and input handling,
and does not become a fullscreen owner. Layer-shell overlays remain above it.
The clipboard is owned by the window while it is open; persistent clipboard
history is a separate clipboard-manager feature.

## Ownership

The file watcher and worker belong to a runtime Service. Shutdown unregisters its
calloop channel and joins the worker after backend teardown. Error children are
reaped without waiting inside the active render loop; teardown terminates and
waits for any remaining owned error child. No GTK event loop or file watcher
callback runs on the compositor rendering thread.

Programmatic run_with_settings uses the same typed model without loading,
creating, or watching the user's Lua file.
