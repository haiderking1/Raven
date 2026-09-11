# Waybar integration issues

This records the startup/Waybar/appearance phase. Implementation details remain
in the [workspace protocol](../README.md),
[startup](../../../runtime/startup/README.md),
[runtime settings](../../../runtime/settings/README.md) and
[appearance](../../../desktop/appearance/README.md) documentation.

## Missing output metadata protocol

Waybar 0.15 exited with "Failed to acquire required resources." Raven already
advertised wl_output, layer shell and ext-workspace-v1, but not the XDG-output
manager required by Waybar. Delegating Smithay's output dispatch did not create
that global.

State::new now constructs and retains OutputManagerState::new_with_xdg_output.
Smithay supplies metadata from Raven's actual Output and handles subsequent
changes. The fix is in production initialization, not a test-only global.
The isolated real Waybar run then mapped and rendered the bar.

## Private session-bus setup and abort cleanup

The isolated run initially pointed Waybar at a nonexistent session bus. Waybar
exited during portal setup before its panel could map. The runner now owns a
private dbus-daemon, socket and configuration without service directories or
activation helpers. An unavailable portal produces an informational warning;
no ambient session service is activated. This was a fixture setup problem, not
a reason to change Raven's session-bus environment.

The original wrapper only removed the private runtime on exit. Timeout drills
exposed detached Xwayland processes and stale X11 paths. Cleanup now stops the
owned command group, checks detached client ownership through pidfds and exact
private-runtime identity, and reaps adopted children. It removes only unchanged
recorded X11 socket/lock paths after process exit and abstract-listener release.
The successful full run required no forced client termination. See the
[isolated-run documentation](../../../runtime/client/tests/waybar/README.md).

## Workspace module configured under the wrong name

The user's bar displayed ext/workspaces, but its settings were still under
hyprland/workspaces. There was no ext/workspaces on-click action. Waybar 0.15
only installs its workspace click handler when an action is configured, so
those clicks sent no activation request.

On the user's explicit request, an ext/workspaces block with on-click set to
activate was added to the personal Waybar config. Other settings were retained;
no bar or compositor restart was performed by the assistant. That personal file
is not part of the repository. The repository's example already includes the
action. Raven does not rewrite Waybar configuration at startup.

## Activation discarded while the mouse button was held

A separate Raven defect remained even with the action configured. Waybar sends
activation on button press. Raven's workspace guard rejected all pointer grabs,
including the ordinary implicit ClickGrab created by that same press.

The earlier isolated click helper delivered press and release before dispatching
the client's activation request. It verified requests and state feedback but
missed this held-button ordering. Its successful result did not establish that
physical clicks worked reliably.

Raven now retains one committed activation from a mapped layer's ordinary
ClickGrab and applies it after release through the existing workspace switch.
It does not remove or bypass the grab. Window selections, custom drag grabs and
keyboard grabs retain their rejection policy. Later intent and relevant resource,
surface, grab and session lifecycle changes cancel deferred work. Details are
in [panel activation](../activation/README.md).

No tests were added or run for this correction, as requested. Formatting,
production library/binary checking, release build and whitespace checks passed.
The frame scheduler, GPU timing and dependencies were unchanged.

## User confirmation and limits

The user initially confirmed the bar and appearance worked but reported that
workspace clicks did not switch. After the Raven correction and the requested
Waybar configuration edit, the user reported that clicks worked.

This is live user confirmation of the reported symptom. It does not independently
validate every cancellation path or measure input latency. Earlier isolated
Waybar/EGL results covered mapping, reservation, workspace protocol feedback,
fullscreen and cleanup; they did not provide physical input or DRM presentation
measurements. Local logs remain under ignored prefromance-inputlag/verification.
Resize transactions and snapshot animations were implemented in the subsequent
[resize work](../../../desktop/resize/integration/README.md). Lua configuration
remains planned. The reported fullscreen flicker has not yet been rechecked live
with that build.
