# On-demand Xwayland

With xwayland-satellite >=0.7 and Xwayland installed, the normal Raven launch
reserves an X display automatically. Satellite starts only when an X11 client
connects. No separate systemd service or manual DISPLAY assignment is needed.

Raven-managed clients retain WAYLAND_DISPLAY and receive the reserved DISPLAY;
terminals and launchers pass those values to their children. Inherited foreign
DISPLAY/XAUTHORITY values are not used for that endpoint. Raven does not mutate
its process-global environment or another session’s systemd environment.

RAVEN_XWAYLAND_SATELLITE=off disables integration. Another value selects the
executable; unset uses xwayland-satellite from PATH. A missing/unsupported binary
or failed reservation logs a warning and leaves native Wayland available.

## Ownership and activation

- Setup runs before the TTY backend acquires input/DRM. A worker performs the
  capability probe, with a three-second exit deadline, and exclusive reservation.
- Filesystem and Linux abstract X sockets stay owned by Raven. Existing locks or
  sockets are skipped, not reclaimed. Cleanup checks directory and inode identity
  before unlinking owned paths. No other session’s display is removed.
- The first connection stays in the kernel backlog. Satellite receives the
  display number and both sockets through -listenfd. CLOEXEC is cleared only on
  those descriptors in the child. The worker stops watching the listening sockets
  while Xwayland accepts connections.
- The worker polls control/listener descriptors and a kernel pidfd. There are no
  periodic idle or child-exit wakeups. A crash triggers bounded 1–30-second retry
  backoff and bounded clearing of stale queued connections before rearming.
- Satellite has an owned process group, including its Xwayland child. The leader
  remains unreaped until group cleanup, protecting against PID reuse. After
  terminating the group, the worker waits on remaining member pidfds before
  releasing sockets. Leader exit alone does not prove Xwayland closed them.
  Shutdown restores the TTY backend before joining the worker or waiting.
- Abrupt process death can leave lock/path entries; Raven deliberately does not
  guess that existing entries are safe to remove. Linux pidfds/procfs are required.

## Protocol dependency

Satellite requires wp_viewporter. Raven delegates it through Smithay’s committed
surface state. The repository pins Smithay 0.7.0 through vendor/smithay with
viewport-hook and retained-buffer mapping/damage fixes. See that package’s
patches/README.md for provenance. No installed registry files are modified.

## Validation boundary

The single explicit integration regression uses the installed satellite, a real
X11 client, Raven’s Wayland event loop, and real EGL scene import on an isolated
headless output. It verifies mapping, committed fullscreen/X11 geometry, unmapping
and socket cleanup. It does not acquire DRM master or exercise an actual game.

Raven now implements relative-pointer and pointer constraints through the capture
module. The separate real-wire capture regression checks delivery and lifecycle.
The X11 regression also verifies transient and parentless splash floating without
changing the main tile. Neither establishes physical game sensitivity, complete
game compatibility or GPU performance.
