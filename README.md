# Raven

A direct-TTY [Wayland](https://wayland.freedesktop.org/) compositor written in Rust on [Smithay](https://github.com/Smithay/smithay) (vendored and patched in-tree).

Built as a personal **tiling** compositor — workspaces, Waybar, Xwayland, and an adaptive frame pipeline — with an emphasis on correct DRM/input behavior rather than feature checklist completeness. Floating windows are not implemented yet.

## Status

Actively used and under development. Single GPU and single output today. Multi-output is intentionally deferred. Expect sharp edges; this is not positioned as a drop-in replacement for Hyprland/Sway.

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

## Build

```bash
cargo build --release --locked -p raven
```

## Run

From a free VT as your login user:

```bash
cargo run --release --locked -p raven
```

Optional: launch an extra client after Waybar’s default startup entry:

```bash
cargo run --release --locked -p raven -- -- foot
```

Frame pipeline (optional):

```bash
RAVEN_FRAME_PIPELINE=adaptive   # default
RAVEN_FRAME_PIPELINE=immediate
RAVEN_FRAME_PIPELINE=deadline
```

### Shortcuts

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
