# Runtime settings

runtime::settings::Settings owns startup, appearance and resize animation values. runtime::run uses
the defaults; run_with_settings accepts another Settings value. Appearance is
validated before backend acquisition and applied before any client launches.
Startup validation preserves individual entry errors for nonfatal launch reports.
Constructing values never starts processes or changes a live compositor.

The default startup command is waybar. An empty StartupPlan.entries disables it.
Waybar reads its own normal configuration; Raven does not replace personal files.
Use its ext/workspaces module for Raven workspace buttons. A minimal separate
example is in protocols/workspace/examples/waybar.jsonc.

Appearance defaults to 8 logical pixels for each gap and a 2-pixel border. Inner
horizontal/vertical gaps, outer edges, border width, and active/inactive RGBA
colors are independent. Appearance::disabled removes both spacing and borders.
State::set_appearance validates atomically and updates the running layout without
restarting startup commands. Fullscreen remains gapless and borderless.

ResizeAnimations defaults to a 200-millisecond cubic ease-out.
ResizeAnimations::from_millis accepts 0 through 2000 milliseconds; zero and
ResizeAnimations::OFF disable visuals without disabling resize transactions.
The duration is private, so a future Lua loader uses the same validated type.

Related documentation:

- [Startup entries and child ownership](../startup/README.md)
- [Appearance geometry, rendering and input](../../desktop/appearance/README.md)
- [Coordinated resize transactions](../../desktop/resize/README.md)
- [Snapshot resize animations](../../desktop/animation/README.md)
- [Resize integration issue and build status](../../desktop/resize/integration/README.md)
- [Waybar workspace protocol](../../protocols/workspace/README.md)
- [Encountered issues and confirmation](../../protocols/workspace/bugs/README.md)

These are the real runtime values a future Lua loader will produce. No Lua
parser, user config file or reload binding is introduced by these settings.
