# Raven TODO

Deferred work, not a release schedule. Start with session locking and screen
sharing; multi-output remains deferred.

## Session locking and idle handling

- [ ] Support a Wayland session locker with secure input and window isolation.
- [ ] Keep the session locked if the locker crashes or disconnects.
- [ ] Add idle notifications and idle inhibition for idle-management tools.
- [ ] Support automatic output power-off and wake-up without losing the lock.

## Screen capture and sharing

- [ ] Add capture protocol support for screenshots and recording.
- [ ] Integrate a compatible desktop portal backend for screen sharing.
- [ ] Verify OBS capture and browser/Discord sharing, including permission handling.

## Monitor configuration

- [ ] Expose resolution and refresh-rate selection in Lua.
- [ ] Add configurable scaling and output transforms.
- [ ] Reject unsupported output settings without losing the working display.
- [ ] Later: support multiple outputs, placement, workspace assignment, and hot-plug.

## Input configuration

- [ ] Add keyboard layouts, variants, options, and layout switching.
- [ ] Add mouse sensitivity and per-device settings.
- [ ] Add touchpad settings, including tapping and natural scrolling.
- [ ] Add touchpad gestures with cancellation and grab handling.

Keyboard repeat, mouse acceleration profiles, and virtual-pointer input are
already supported.

## Window rules

- [ ] Add generic application/window matching rules.
- [ ] Support initial floating mode and workspace placement.
- [ ] Define rule ordering and how reloads affect existing windows.

## Reliability and regression coverage

- [ ] Fix the existing rendering-test compilation errors and run the library suite.
- [ ] Verify suspend/resume and repeated VT switching with held input.
- [ ] Verify input-device unplug/reconnect and settings reapplication.
- [ ] Test output unplug and device-failure handling without hangs or crashes.
- [ ] Verify idle power behavior and wallpaper transitions in a live session.

Preserve the approved fullscreen, floating resize, dragging, popup focus, cursor,
and configuration-reload behavior while adding these features. The closed
fullscreen-entry flash investigation is not reopened by this checklist.

## Documentation

- [ ] Update stale README claims, including the claim that floating is unimplemented.
- [ ] Keep the offline handbook aligned with supported settings and protocols.
