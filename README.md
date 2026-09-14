# Raven

A direct-TTY [Wayland](https://wayland.freedesktop.org/) compositor written in Rust on [Smithay](https://github.com/Smithay/smithay) (vendored and patched in-tree).

Built as a personal **tiling** compositor — workspaces, Waybar, Xwayland, and an adaptive frame pipeline — with an emphasis on correct DRM/input behavior rather than feature checklist completeness. Floating windows are not implemented yet.

## Status

Actively used and under development. Single GPU and single output today. Multi-output is intentionally deferred. Expect sharp edges; this is not positioned as a drop-in replacement for Hyprland/Sway.

Deferred features and validation work are tracked in the [TODO list](docs/planning/TODO.md).

## Layout

    crates/raven/     compositor binary + library
    vendor/smithay/   vendored Smithay (workspace [patch.crates-io])

Main modules under `crates/raven/src/`:

- `backend/` — DRM/KMS, libinput, libseat, rendering and frame scheduling
- `desktop/` — tiling, fullscreen, workspaces, layers, animations
- `input/` — keyboard, pointer, timing
- `protocols/` — Wayland protocol handlers
- `runtime/` — startup, clients, settings, event loop
- `state/` — compositor state

## Requirements

- Linux with an **active** seat session (`logind` or `seatd`)
- Run as your normal login user on a real VT (not root)
- Rust toolchain from `rust-toolchain.toml` (pinned nightly)
- Typical Smithay/DRM stack (libseat, libinput, GBM/GLES, etc.)
- GTK4 development files and pkg-config for the separate configuration error window

## Build

```bash
cargo build --release --locked -p raven
```

## Run

From a free VT as your login user:

```bash
cargo run --release --locked -p raven
```

Optional: launch an extra client alongside the configured startup applications:

```bash
cargo run --release --locked -p raven -- -- foot
```

Developer-only frame-pipeline diagnostics:

```bash
RAVEN_FRAME_PIPELINE=adaptive   # default
RAVEN_FRAME_PIPELINE=immediate
RAVEN_FRAME_PIPELINE=deadline
```

### Wallpapers

Layer-shell wallpaper tools such as awww and swaybg work through Raven's existing
background layers. No wallpaper daemon is forced on users. See
[wallpaper setup and compatibility checks](crates/raven/src/desktop/layers/wallpapers/README.md).

### Configuration

Raven creates `~/.config/raven/raven.lua` on first startup, or uses
`$XDG_CONFIG_HOME/raven/raven.lua` when configured. Save to hot reload. Invalid
changes keep the working settings and open an error window with a Copy error
button. `Super+Shift+R` reloads manually by default.

Open the [HTML configuration handbook](docs/config/index.html) for step-by-step
examples, argument syntax, and troubleshooting. It works offline.
The [Lua configuration guide](crates/raven/src/runtime/config/README.md) also
documents the implementation and reload behavior.
Check a configuration without starting a compositor:

```sh
./target/debug/raven --check-config
```

### Default shortcuts

| Binding | Action |
| --- | --- |
| Super+Q | Launch Foot |
| Super+D | Launch Fuzzel |
| Super+C | Close focused window |
| Super+F | Toggle fullscreen |
| Super+1…9 / 0 | Switch workspace (0 → 10) |
| Super+Shift+number | Move focused window to workspace |
| Super+Shift+Q | Exit Raven |
| Ctrl+Alt+F1…F12 | Switch VT |
