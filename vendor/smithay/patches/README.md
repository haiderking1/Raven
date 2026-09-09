# Raven's Smithay 0.7.0 patch

## Source

Copied from the installed crates.io source package:

`/home/soka/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/smithay-0.7.0`

- Upstream: https://github.com/Smithay/smithay
- Package version: **0.7.0**, unchanged in both upstream manifests.
- Published archive SHA-256 (from Raven's original lockfile):
  `740cea6927892bc182d5bf70c8f79806c8bc9f68f2fb96e55a30be171b63af98`
- Packaged VCS revision: a166cf4c94b5aedc332a65aa1dd753e8148829c3,
  recorded in the package's .cargo_vcs_info.json (not obtained from Git history).
- License: MIT; LICENSE.txt and all upstream source notices are retained.

The upstream package layout, examples, benches, build files, Cargo.toml,
Cargo.toml.orig, and package Cargo.lock are preserved. Only .cargo-ok and
.cargo_vcs_info.json were omitted as registry bookkeeping; the latter's provenance
is recorded above. The registry source was not edited.

## Repository integration

The root Cargo.toml keeps smithay = "=0.7.0", excludes vendor/smithay from the
workspace, and adds a crates.io path patch pointing here. The root Cargo.lock
changes only by removing Smithay's registry source and checksum. Its version and
dependency list, every other resolved package, and nightly-2026-09-06 are unchanged.
The package-local lockfile is upstream provenance, not Raven's resolution.

## Source changes

The viewport portion modifies two upstream Rust files, each with a private helper.
The additional capture changes are documented in [pointer-constraints.md](pointer-constraints.md).
They add motion dispatch hooks and committed constraint lifecycle handling while
retaining version 0.7.0.

Viewport changes:

- src/wayland/viewporter/mod.rs and new commit_hook.rs: register the existing
  viewport pre-commit hook once per surface using a separate marker. Renderer
  validation may create the viewport-object slot first without suppressing hook
  registration. Duplicate-object rejection, destruction's pending-state reset,
  recreation, and the existing validation hook are unchanged. This implements the
  intent of prefromance-inputlag/dependency-patches/viewport-hook-registration.patch.
- src/backend/renderer/utils/wayland.rs and new wayland/surface_commit.rs: apply
  committed scale/transform and rebuild the surface view with or without a new
  attachment. Validate viewport bounds against the resulting transformed,
  integer-scaled buffer dimensions. Drain both surface-space and buffer-space
  damage, using the resulting view for conversion and clipping to buffer bounds.
  Mapping/view/dimension changes add full buffer damage, advancing the existing
  damage counter even when a crop or transform changes pixels without changing
  destination geometry. Rebuild opaque regions on mapping changes as well.

Retained pixel damage clears cached textures so normal renderer imports refresh
SHM content. Mapping-only commits keep those textures. Neither path replaces or
drops the retained Buffer, takes acquire/release points, or manufactures a buffer
assignment. New-buffer ownership and explicit synchronization remain in the
original assignment branch. No acquire hooks are added or replayed. Removing a
buffer retains the original reset/release behavior and clears committed damage;
failed buffer-dimension lookups and bufferless commits also discard damage
rather than carrying it into a later attachment. A viewport destination alone
does not create content.

The renderer still runs through on_commit_buffer_handler after cached state is
applied. Synchronized children remain deferred until the parent transaction is
applied; the helper reads current state, never pending state. Repeated tree walks
with unchanged state and drained damage do not advance the damage counter.

## Verification

Passed on the repository's unchanged nightly:

- cargo check --workspace --offline
- cargo check --workspace --all-targets --offline --locked
- cargo fmt --all --check --verbose (only Raven targets, no upstream targets)
- Direct rustfmt of the four changed/new Rust modules, with skip_children=true.
- Parsed lockfile comparison: no package changes besides Smithay's source/checksum.
- Byte comparison with the registry copy: only the two upstream Rust files differ;
  all other upstream files are identical, apart from the two omitted markers.

Compilation reports eight warnings in unchanged upstream source (unused imports,
an atomic-method deprecation, and a function-pointer cast). No tests were added or
run for this patch. The main agent owns the focused end-to-end regression; these
compilation checks do not establish runtime viewport behavior or performance.
