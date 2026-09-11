# Raven

A **Wayland compositor** written in **Rust**, built on [Smithay](https://github.com/Smithay/smithay).

Early / experimental — the kind of systems project that shows low-level Rust, not a polished daily driver yet.

## What’s in here

Workspace crate layout:

```text
crates/raven/     compositor binary + library
vendor/smithay/   vendored Smithay (patched via workspace)
```

Source is split by responsibility under `crates/raven/src/`:

- `backend/` — display / rendering backend
- `desktop/` — desktop shell pieces
- `input/` — input handling
- `protocols/` — Wayland protocols
- `runtime/` — event loop / runtime wiring
- `state/` — compositor state

## Requirements

- Rust toolchain (see `rust-toolchain.toml`)
- A Linux environment suitable for Wayland / Smithay development

## Build

```bash
cargo build -p raven
```

Run (when your environment is set up for a compositor):

```bash
cargo run -p raven
```

## Status

Work in progress. Expect breakage. Useful as a portfolio signal for **Rust + Wayland systems programming**.
