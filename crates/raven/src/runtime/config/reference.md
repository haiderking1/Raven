# Lua settings reference

## Commands and bindings

```lua
raven.commands { terminal = { "foot" }, launcher = { "fuzzel" } }
raven.bind("Super+Return", "terminal")
raven.bind("Super+E", { spawn = { "nautilus", "--new-window" } })
raven.bind("Super+2", "workspace", 2)
raven.bind("Super+Shift+2", "move_to_workspace", 2)
raven.bind("Ctrl+Alt+F2", "vt", 2)
raven.unbind("Super+Q")
```

Modifiers are Super/Logo, Shift, Ctrl/Control, and Alt. Modifier matching is
exact, excluding lock state. The final component is an XKB keysym name, such as
Return, space, F2, or XF86AudioRaiseVolume. Use unshifted symbols with an explicit
Shift modifier. A plus key can be named `plus`.

Actions without parameters: terminal, launcher, close, fullscreen, floating, screenshot,
next_app, previous_app (both require Alt in the binding),
quit, reload. workspace and move_to_workspace take a number from 1 through 10.
vt takes a number from 1 through 12. Spawn takes a literal argv array.

Binding the same chord replaces it. Unmentioned default bindings remain enabled.
`raven.clear_bindings()` removes all configured bindings, including quit and VT
bindings; add replacements afterward. Escape during a compositor-owned window
drag remains its cancellation action. Reload preserves the press/release
disposition of keys already held down.

Terminal and launcher executables must resolve through Raven's PATH, or through
the supplied absolute/relative path, and pass executable-permission checks.
Relative paths use Raven's working directory, just like launching. A missing or
non-executable command rejects the whole reload and keeps the working settings.
Raven never autocorrects names or executes commands during validation.

This checks only the first argv entry. It cannot verify commands inside a shell
or wrapper, missing script interpreters/libraries, or a program's later behavior.
Optional startup entries are not subject to this rejection rule; actual startup
launch failures remain logged without blocking other entries.

## Startup

```lua
raven.startup {
    { "waybar" },
    { "awww-daemon" },
    { argv = { "foot", "--title", "Notes" },
      cwd = raven.env("HOME"), env = { EDITOR = "nvim" } },
}
```

The list replaces the default startup plan. An empty list disables startup apps.
It runs once per session, not on reload. Invalid argv/cwd/env reject the candidate;
a program that cannot be launched at session startup is logged without stopping
other entries. Display routing belongs to Raven and overrides entry environment
values. Reloading a changed startup list affects the next session only.

## Appearance and animations

```lua
raven.appearance {
    inner = { horizontal = 8, vertical = 8 },
    outer = { top = 8, right = 8, bottom = 8, left = 8 },
    border = { width = 2, active = "#5c7a94", inactive = "#333840" },
}
raven.animations { resize_ms = 200 }
```

An integer inner or outer value sets all of its axes/edges. Nested fields are
optional and update the candidate defaults. Gaps and border widths are logical
pixels from 0 through 65535, further constrained by output geometry. Border
colors accept #RRGGBB or #RRGGBBAA. Resize duration is 0 through 2000 milliseconds;
zero disables the animation, not synchronization.

## Cursor

```lua
raven.cursor { theme = "Adwaita", size = 24 }
```

Theme and positive logical-pixel size reload together. Raster Xcursor search
paths, inheritance, and aliases apply. Assets are prepared before publication;
loading a cursor must succeed before any candidate setting is applied. The
selected theme can inherit the system default through normal Xcursor fallback.
Existing client-provided cursors are not clamped or rewritten. SVG-only cursor
themes remain unsupported. Unchanged cursor settings preserve the active cache
and animation phase.

## Workspaces

```lua
raven.workspaces { show = "occupied", persistent = { 1, 2, 3, 4, 5 } }
-- Or: raven.workspaces { show = "all" }
```

Occupied mode keeps occupied, persistent, and active workspaces visible.
Persistent numbers can be any subset of 1 through 10. These settings do not
change workspace count, identities, placement, or shortcut availability.
Waybar's ext/workspaces must use ignore-hidden=true to honor Raven's Hidden hints;
ignore-hidden=false still tells Waybar to show every workspace.

## Input

```lua
raven.input {
    keyboard = { repeat_rate = 30, repeat_delay = 300 },
    mouse = { accel_profile = "flat" },
}
```

Keyboard repeat rate accepts 0 to 1000 repeats per second, default 25. Use 0
to disable repeat. Repeat delay accepts 0 to 60000 milliseconds, default 400.
Values are sent to Wayland clients on reload and when they bind a keyboard;
applications may implement their own repeat behavior. Held-key ownership is
unchanged.

Mouse profiles are "default", "flat", and "adaptive". The default restores
each mouse’s libinput default. Flat disables speed-dependent acceleration,
not libinput’s constant scaling. Adaptive enables speed-dependent acceleration.
Pointer speed is unchanged. Only udev-tagged mice with configurable profiles
are affected; touchpads and pointing sticks are excluded.

Connected mice are checked before applying a reload; unsupported profiles
reject the whole candidate. Device update failures restore earlier updates
before rejecting the candidate. Newly connected mice receive the active
profile; failures leave their existing profile and are logged. Devices
without configurable acceleration profiles are left unchanged.
