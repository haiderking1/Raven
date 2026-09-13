# Wallpapers

Raven supports wallpaper clients through wlr-layer-shell. There is no wallpaper
application allowlist, namespace special case, built-in image loader, or forced
daemon. Use awww, swaybg, or another compatible layer-shell client.

## Using awww

From a terminal inside Raven, start the daemon:

```sh
awww-daemon &
```

Wait until `awww query` reports your output at its configured resolution, then
select an image:

```sh
awww query
awww img /absolute/path/to/wallpaper.png
```

awww owns image decoding, transitions, animation, and its cache. Use its own
options to choose transitions or outputs. `awww kill` stops its daemon. Raven
does not override those preferences.

## Using swaybg

Run this instead of awww when a static wallpaper is enough:

```sh
swaybg -i /absolute/path/to/wallpaper.png -m fill &
```

A solid color is also supported:

```sh
swaybg -c '#1e1e2e' &
```

Choose one wallpaper daemon unless you intentionally want overlapping background
surfaces. Raven applies normal layer ordering rather than picking a preferred app.

## Session startup

Raven does not launch a wallpaper daemon by default. Its existing extra-client
argument can start whichever tool you choose alongside the default Waybar entry:

```sh
./target/debug/raven -- awww-daemon
# Or, in a separate session:
./target/debug/raven -- swaybg -i /absolute/path/to/wallpaper.png -m fill
```

For session startup, configure arbitrary argv entries in raven.lua:

```lua
raven.startup { { "waybar" }, { "awww-daemon" } }
```

This replaces the startup list and runs once per session, never on hot reload.
It uses the generic StartupPlan, not an app-specific wallpaper setting.
See [Lua configuration](../../../runtime/config/README.md) and
[startup ownership](../../../runtime/startup/README.md).
Daemon readiness and image-selection commands must be coordinated by the chosen
tool or a user-owned script, not a fixed sleep inside Raven.

## Compositor behavior

Background and Bottom layers render behind ordinary windows and remain attached
to the output across workspace switches. Wallpaper clients normally anchor to
all four edges and set exclusive zone -1, covering the complete output rather
than reserving tile space or shrinking around a panel.

Raven honors the client's input region and keyboard-interactivity request. Both
awww and swaybg use an empty input region; awww explicitly requests no keyboard
interactivity, and swaybg leaves the protocol default of none. Raven does not
force every Background surface to be input-transparent.

Buffer scale and wp_viewporter determine sampling. The normal SHM import, damage,
frame-callback, and release paths handle wallpaper surfaces. A static image does
not require a new wallpaper animation loop. Wallpaper transitions use ordinary
client commits and compositor frame admission.

Existing fullscreen layer visibility, frame pacing, and synchronization remain
unchanged. Raven currently supports one output; this feature does not add
multi-output management or fractional output configuration.

## Compatibility checks

On the running Raven session at commit 943ad23:

- Installed awww-daemon 0.12.1 accepted a PNG and a short fade request. It reported
  DisplayPort-2 at 1920x1080, scale 1, and the selected image. Its buffer pool
  advanced to three buffers and the daemon remained alive.
- Upstream swaybg a59ea3d received and acknowledged a 1920x1080 layer configure,
  then attached a 1x1 SHM solid-color buffer using its viewport path. It remained
  alive without a protocol error in the captured exchange.

The awww source comparison used revision 25ea4fd. Its required compositor, SHM,
layer-shell, and viewporter globals are already available in Raven; fractional
scale is optional. No compositor patch was necessary for these checks.

The probes used temporary caches and owned processes, which were stopped
afterward. No startup or personal configuration was changed. These checks prove
client setup and submission, not pixel-level visual correctness or idle power
usage. Visual checks still include image fit, transitions, workspace switches,
fullscreen entry/exit, and focus/input behavior.
