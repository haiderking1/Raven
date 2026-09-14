# Native screenshots

Print and Super+Shift+S run the `screenshot` action. A frozen compositor-owned
image backs an area selector. Drag, release, and press Enter or Space to save a PNG and
publish an image/png clipboard selection. Escape or right-click cancels preview.
No capture protocol, shell command, external selector, or helper application runs.

## Ownership

The screenshot renderer draws the normal scene into an upright private texture
after ordinary output submission. This includes layer shells and the app switcher,
but excludes cursors and the screenshot controls. Source render elements remain
owned until the copy fence completes, including any nested resize snapshots.
Cancellation must wait for that specific copy if its inputs are still in use;
there is no blanket GPU wait. The image is not a Wayland surface and claims no
client presentation feedback or zero-copy status.

Opening dims the frozen texture by 50% without a selection frame or help panel.
Dragging reveals the selected pixels with a two-logical-pixel outline outside
their bounds. Cursor-only movement does not dirty the frozen image.
Confirmation reads only the selected rectangle into a private PBO. A worker waits
on a fence placed after readback, then wakes the event loop. Mapping happens only
after readiness and only while the backend is active. PNG compression and disk
writes run off-thread. Confirmed saves finish before the encoder is dropped.

## Input

All keys still pass through XKB. The first press owns its corresponding release
through cancellation, modifier release, and binding reload. Previously forwarded
keys keep their release; newly pressed modal controls are intercepted. Input that
arrives before capture finishes updates the pending selection instead of being
dropped. Capturing Alt+Tab suppresses Alt-release activation and closes the app
switcher only after its pixels have been copied.

Physical and virtual pointers share selector delivery and button ownership.
Client motion and scrolling stop during selection; existing pointer constraints
are retained and the original cursor position is restored on exit. Session
suspension, output changes, workspace changes, and new grabs cancel preview.

## Output

Pictures comes from the XDG user directory, with HOME/Pictures as fallback. PNGs
are written to private temporary files and published with no-overwrite rename.
A failed save does not discard an encoded clipboard image. Clipboard offers own
their PNG bytes, so replacing the clipboard cannot alter an ongoing transfer.
Pipes and sockets use bounded nonblocking writes; regular file recipients use a
worker. Transfer counts and retained payload sizes are bounded.

## Checks

`cargo test --locked -p raven --lib screenshot` covers actual XKB controls, selection
bounds, pending input, cancellation, PNG pixels, private files, and real Wayland
clipboard offers with slow readers and file-backed recipients.

`cargo test --locked -p raven --lib screenshot -- --ignored` checks the GPU path,
all eight output transforms, fractional/integer scaling, cropping, cursor
exclusion, initial shading, and unobstructed selection. These are headless checks, not
validation of live KMS presentation or interactive behavior.
