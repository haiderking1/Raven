# Application switcher

Alt+Tab opens a compositor-native application strip. App IDs define groups;
missing IDs remain separate. A snapshot of focus history orders groups and their
windows. Preview never changes keyboard focus. Confirmation activates the most
recent mapped window of the selected app, switching workspaces when needed.

A target hidden by another fullscreen window waits for the normal fullscreen exit
transaction. No resize cohort, acquire fence, or presentation owner is bypassed.
Workspace restoration is excluded from recent-app history until the requested
window actually receives focus.

## Input ownership

All keys still pass through Smithay/XKB. Alt release confirms only after XKB has
processed the release; its original press/release ownership is preserved. Newly
pressed keys during preview are intercepted, but previously forwarded keys keep
their release. Tab, arrows and Escape are switcher controls. Held Tab repeats
using the configured delay/rate; the repeat source is removed on key release,
cancellation or suspension. Window grabs and exclusive keyboard layers prevent
opening. Session suspension and workspace changes cancel preview.

Pointer motion during preview does not reach applications. Clicks that dismiss
the strip retain ownership of their later releases. The pointer uses the themed
default cursor temporarily without overwriting the client cursor.

## Artwork

A bounded latest-request worker resolves desktop entries, icon-theme inheritance,
and rasterizes Cairo/Pango artwork. It initializes no GTK widgets or display
connection. Desktop entries are never executed. Icon and metadata caches are
bounded; late results cannot reopen a closed switcher. Long lists use a viewport
that keeps the selected icon visible rather than shrinking icons indefinitely.

The TTY renderer draws the opaque dark panel artwork without backdrop blur or
framebuffer capture. The overlay uses normal hardware-plane eligibility and
requests frames only for input, artwork completion, and normal scene updates.
It does not change the frame scheduler.

## Verification

- `cargo test --locked -p raven --test switcher`: grouping, selection, overflow,
  and keyboard press/release ownership.
- `cargo test --locked -p raven --lib desktop::switcher`: real-window focus,
  workspace selection, XKB Alt release, and artwork pixels.

Live session appearance, fullscreen switching, and hardware pacing still require
interactive verification. The older library fixture suite also has outstanding
failures; fixing its stale call signatures does not make that suite pass.
