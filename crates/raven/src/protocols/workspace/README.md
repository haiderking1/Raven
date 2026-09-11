# Waybar workspaces

Raven implements version 1 of `ext_workspace_manager_v1` for stock Waybar 0.15's
`ext/workspaces` module. The installed
`/usr/share/man/man5/waybar-ext-workspaces.5.gz` documents this module even though
its generated heading still says wlr workspaces.

Use [examples/waybar.jsonc](examples/waybar.jsonc) as a separate configuration or
merge its module entry into your own config. It does not change personal settings.
The module needs its own `"ext/workspaces"` settings block containing
`"on-click": "activate"`. Settings under `"hyprland/workspaces"` do not apply.
Waybar 0.15 does not install a workspace click handler without a configured action.
No custom CSS, IPC, polling, or Waybar patches are needed.
Encountered failures, fixes and user confirmation are in the
[integration issue history](bugs/README.md).

This is not `wlr/workspaces`. It also does not implement `wlr/taskbar`.
The taskbar needs a separate foreign-toplevel protocol to list windows;
workspace handles are not window handles.

## Published state

- Ten workspaces named `1` through `10`, with session-stable IDs
  `raven-workspace-1` through `raven-workspace-10` and one-dimensional coordinates
  `0` through `9`.
- One stable workspace group spanning Raven's current output. It remains present
  with no outputs when `State::output` is `None`.
- Exactly one active workspace. Only the Activate capability is advertised.
  Create, deactivate, remove, and assign requests are ignored.
- Output membership includes every live `wl_output` object belonging to the
  manager's client, including objects bound later. Removing or replacing the
  output emits leave for its live objects. Released objects are forgotten without
  using a destroyed object as an event argument.

Each manager owns its pending activation, even when a client binds twice.
The last activate request before that manager's commit wins. Window selections,
DnD and keyboard grabs still deny switching. Those denied requests are consumed,
not replayed on release. The reply always reports the requested workspace's
actual state and ends with done. Other managers receive changed states only.

Waybar sends activation on button press. An ordinary Smithay ClickGrab on a
mapped layer-shell surface therefore defers the committed activation until its
release. Raven keeps one such committed intent globally, with weak manager,
workspace and surface handles. Later committed activation replaces it, including
a request to stay on the current workspace. A no-op commit does not replace it.
The existing post-dispatch refresh applies it through `State::switch_workspace`
after the original surface receives its release. This also keeps keyboard focus
and fullscreen bar hiding from changing partway through the click.

Workspace keyboard intent, new drag/popup grabs, VT suspension, layer unmap or
removal, manager stop/disconnect and target handle destruction cancel deferred
activation. A changed grab identity or detached click surface also cancels it.
There is no timer, per-client queue, synthetic release or Waybar-name exception.

Stop immediately sends finished,
drops pending work and snapshots, and makes the remaining child handles inert.
Destroy and disconnect callbacks remove tracked resources. Tracking uses weak
resources and object IDs, never retained clients.

## Runtime integration

State initialization registers both ext-workspace-v1 and Smithay's XDG-output
manager. XDG-output provides the logical geometry and output identity required
by Waybar; Smithay updates it from the same Output used by Raven's desktop.

The runtime refreshes workspace publication after desktop reconciliation and
before rendering/client flush, and after installing the output. OutputHandler
marks late wl_output binds dirty without querying output state inside its hook.
Unchanged active/output identity and no new binds return before enumerating
subscriptions or allocating snapshots. There is no timer or input-rate polling.
Protocol commits publish their own acknowledged results immediately.

Isolated server loops use the same refresh after desktop reconciliation. Output
replacement/removal must update State::output and desktop mappings before refresh.

## Regression

`cargo test --locked -p raven workspace_wire_transactions_and_lifecycle`

One socket-level test uses the existing desktop Fixture. It checks initial
identity, properties and membership, manager-local atomic commit, successful and
grab-blocked switches, external changes, late/repeated output binds, output
removal and release, stop, child destruction, and disconnect cleanup.

The existing real X11/EGL integration also launches installed Waybar on a private
headless output and private session bus. Its actual buttons switch workspace 2
then 1 and redraw after state/done. A 32px panel reservation resizes the tiled
client; fullscreen hides the bar and removes owner gaps/borders. No active session
or personal Waybar configuration is used.
