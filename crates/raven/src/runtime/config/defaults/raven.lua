-- Raven reads this file on startup and reloads it when you save.
-- Invalid changes keep your last working configuration.

raven.commands {
    terminal = { "foot" },
    launcher = { "fuzzel" },
}

-- Startup runs once per session, never again on reload.
raven.startup {
    { "waybar" },
    -- { "awww-daemon" },
    -- { "swaybg", "-i", raven.env("HOME") .. "/Pictures/wallpaper.png", "-m", "fill" },
}

raven.appearance {
    inner = { horizontal = 8, vertical = 8 },
    outer = { top = 8, right = 8, bottom = 8, left = 8 },
    border = {
        width = 2,
        active = "#5c7a94",
        inactive = "#333840",
    },
}

raven.animations { resize_ms = 200 }
raven.cursor { theme = "default", size = 24 }

-- Active workspaces always remain visible. These hints do not remove workspaces.
-- Waybar's ext/workspaces must use ignore-hidden=true to honor them.
raven.workspaces { show = "occupied", persistent = {} }
-- raven.workspaces { show = "occupied", persistent = { 1, 2, 3, 4, 5 } }
-- raven.workspaces { show = "all" }

-- These replace the matching built-in bindings. Unmentioned bindings stay enabled.
raven.bind("Print", "screenshot")
raven.bind("Super+Shift+S", "screenshot")
raven.bind("Alt+Tab", "next_app")
raven.bind("Alt+Shift+Tab", "previous_app")
raven.bind("Super+Q", "terminal")
raven.bind("Super+D", "launcher")
raven.bind("Super+C", "close")
raven.bind("Super+F", "fullscreen")
raven.bind("Super+V", "floating")
raven.bind("Super+Shift+Q", "quit")
raven.bind("Super+Shift+R", "reload")

for workspace = 1, 10 do
    local key = tostring(workspace % 10)
    raven.bind("Super+" .. key, "workspace", workspace)
    raven.bind("Super+Shift+" .. key, "move_to_workspace", workspace)
end

-- raven.bind("Super+Return", { spawn = { "foot" } })
-- raven.unbind("Super+Q")
-- raven.clear_bindings() -- removes all bindings, including quit and VT bindings
-- raven.bind("Ctrl+Alt+F1", "vt", 1)

-- Repeat delay is in milliseconds; rate is repeats per second.
raven.input {
    keyboard = { repeat_rate = 25, repeat_delay = 400 },
    mouse = { accel_profile = "default" }, -- "flat" disables acceleration
}
