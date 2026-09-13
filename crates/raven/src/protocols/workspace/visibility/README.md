# Workspace display choices

Raven keeps all ten workspace handles and shortcuts available. Empty, inactive
workspaces carry the ext-workspace Hidden flag unless they are persistent.
The active workspace always stays visible. Occupancy counts mapped windows,
including floating windows and windows covered by fullscreen.

## Occupied and active workspaces

Run Raven normally. Waybar's ext/workspaces module defaults to ignore-hidden=true:

```json
"ext/workspaces": {
  "on-click": "activate",
  "ignore-hidden": true
}
```

## All workspaces

Set "ignore-hidden": false in Waybar's "ext/workspaces" block. This shows all ten
handles regardless of Raven's hidden hint. Settings under "hyprland/workspaces"
do not apply to this module.

## Persistent workspace choices

Set RAVEN_WORKSPACE_PERSISTENT when starting Raven:

```sh
RAVEN_WORKSPACE_PERSISTENT=1,2,3,4,5 ./target/debug/raven
```

With ignore-hidden=true, workspaces 1 through 5 remain visible even when empty.
Other occupied workspaces and the active workspace also appear. Any subset from
1 through 10 is accepted, for example 1,3,7. Whitespace and duplicate numbers are
accepted. An unset or empty value means no persistent workspaces. Invalid entries,
empty entries inside a list, and numbers outside 1 through 10 stop startup with
an error instead of silently selecting another policy.

This setting is read at startup. It changes panel visibility hints, not workspace
count, window placement, or shortcut access. Raven does not edit Waybar settings.
A panel can override Hidden with its own display policy.

## Publication

Initial binds receive the current flags. Refresh recomputes occupancy after
normal desktop reconciliation, so opening, closing, moving, mapping, or unmapping
a window updates the affected handles without requiring a workspace switch.
Each subscription receives only changed flags followed by one done event.
Activation acknowledgements report the actual flags even when a grab denies a
switch. IDs and handles remain stable throughout visibility changes.
