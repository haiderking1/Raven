# Workspace display choices

Raven keeps all ten workspace handles and shortcuts available. Empty, inactive
workspaces carry the ext-workspace Hidden flag unless they are persistent or
show-all mode is selected. The active workspace always remains visible.
Occupancy counts mapped windows, including floating windows and windows covered
by fullscreen.

## Lua settings

```lua
-- Occupied and active workspaces:
raven.workspaces { show = "occupied", persistent = {} }

-- Keep 1 through 5 visible, plus other occupied/active workspaces:
raven.workspaces { show = "occupied", persistent = { 1, 2, 3, 4, 5 } }

-- Show every workspace:
raven.workspaces { show = "all" }
```

Use one of these choices in raven.lua. It reloads without restarting Raven.
Persistent numbers may be any subset from 1 through 10. Invalid numbers reject
the entire candidate instead of changing part of the configuration.
These Lua settings replace RAVEN_WORKSPACE_PERSISTENT.

See the [configuration guide](../../../runtime/config/README.md).

## Waybar

Waybar's ext/workspaces module defaults to ignore-hidden=true. Keep that setting
to honor Raven's visibility hints. ignore-hidden=false explicitly tells Waybar
to show all ten handles regardless of those hints. Settings under
hyprland/workspaces do not apply to ext/workspaces.

Raven never edits Waybar settings. Visibility hints do not change workspace
count, window placement, identities, or shortcut access.

## Publication

Initial binds receive current flags. Refresh recomputes occupancy after normal
desktop reconciliation, so opening, closing, moving, mapping, or unmapping a
window updates affected handles without requiring a workspace switch. Lua
visibility changes use the same publication path.

Each subscription receives changed flags followed by one done event. Activation
acknowledgements report actual flags even when a grab denies a switch. IDs and
handles remain stable throughout visibility changes.
