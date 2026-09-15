# Window management

Minimization preserves the mapped application and workspace assignment, removes
its tiled slot from layout, and excludes it from rendering and input. Floating
placement is retained. Restoring reinserts the saved tile position, bounded by
the current stack, or restores the existing floating placement. Parent
minimization also hides descendants; a late dialog cannot steal focus from a
hidden parent. Explicit restoration of a descendant restores its ancestor chain.

Fullscreen minimization requests the ordinary fullscreen exit and waits for its
ownership transaction before hiding the window. Taskbar activation and fullscreen
requests reuse the switcher's transactional focus path rather than bypassing it.

## Restore route

The standard wlr foreign-toplevel-management protocol, versions 1 through 3,
provides taskbars with live handles and close, activate, minimize, maximize, and
fullscreen requests. State changes are grouped with done events. Closed handles
cannot control a remapped window. Parent events require version 3; fullscreen
state and requests require version 2.

Minimization requires a restore-capable taskbar subscription for that window.
XDG capabilities update when this availability changes. Stopping a manager leaves
its existing handles usable but does not make new windows restorable. Destroyed
handles do not count. Losing a window's last restore route reveals it without
forcing focus away from another visible application.

The protocol does not prove that a client has drawn a button; it establishes the
standard control connection. A malicious or broken connected taskbar can still
fail to provide its UI. No native Dock is implemented here.

## Maximization

Maximize uses the layer-reserved workarea without tiling outer gaps, respects
client size constraints, and saves the previous floating placement or tiled slot.
It uses the existing resize batches and client configure/acknowledge/commit path.
Pre-map requests wait for the initial commit. Automatic dialog classification is
retained for restoration. Maximizing or unmaximizing a minimized window does not
make it visible or change its independent minimize request.

A manual move/resize leaves maximized mode at its displayed floating geometry;
it must not snap back on the next arrange pass. Floating-toggle while maximized
first restores the prior mode. Fullscreen and maximize remain distinct states.

## Controls and limits

Lua actions maximize and minimize are available without new default shortcuts.
A compatible taskbar's activate request restores and focuses a window, including
on another workspace. Alt+Tab selects the most recent non-minimized window in an
app; minimized-only apps remain listed but are not automatically restored.

Traffic lights, native Dock UI, cooperative custom-header integration, green-button
hover menus, and macOS-style minimize animations are not part of this foundation.
The existing frame scheduler and presentation ownership are unchanged; this does
not establish live hardware performance equivalence.

## Verification

The management tests run production Wayland handlers with the real resize
coordinator, acknowledge configure events, and commit matching SHM buffers. They
cover placement restoration, focus, transient visibility, fullscreen transitions,
legacy taskbar handles, stop/destroy lifetime, and a separate taskbar connection
closing while application clients remain alive.

Run cargo test --locked -p raven --lib desktop::management. The older shared
fixture suite still has unrelated missing-coordinator failures.
