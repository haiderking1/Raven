# Lua configuration

For a browser-friendly walkthrough, open the
[offline HTML handbook](../../../../../docs/config/index.html).

Raven loads `$XDG_CONFIG_HOME/raven/raven.lua`, or `~/.config/raven/raven.lua`
when XDG_CONFIG_HOME is unset or relative. First startup creates a documented
default file only if no file exists. Existing configuration is never overwritten.

Edit and save the file to reload. The default manual reload binding is
`Super+Shift+R`. Check a file without starting a compositor or opening a window:

```sh
./target/debug/raven --check-config
./target/debug/raven --check-config /path/to/raven.lua
```

Use ordinary Lua functions, loops, variables, tables, and the `raven` API. The
configuration starts from built-in defaults on every evaluation; deleting an
option therefore restores its default rather than retaining a stale value.

- [Settings reference](reference.md)
- [Reload, failure, and process ownership](reload.md)
- [Default configuration](defaults/raven.lua)

Lua controls keybindings, launch commands, startup applications, appearance,
resize animation duration, cursor theme/size, and workspace visibility. Frame
pacing, acquire fences, and presentation ownership are internal compositor
mechanisms, not Lua settings. Output management and mouse drag bindings are not
configurable in this version.

Lua is evaluated with table, string, math, UTF-8, and base language facilities.
Use `raven.env("HOME")` to read an environment value and `raven.config_dir` to
refer to the configuration directory. OS commands, package loading, file-loading
functions, and Lua callbacks in input handlers are deliberately unavailable.
Record startup or spawn argv instead; evaluation must not launch processes.
There is no implicit shell, tilde expansion, or environment expansion in argv.

The runtime configuration is authoritative. Move older XCURSOR_THEME,
XCURSOR_SIZE, and RAVEN_WORKSPACE_PERSISTENT preferences into the Lua settings.
Waybar still controls its own filtering and style. Raven does not edit its files.

The error window uses GTK4 in a separate invocation of the same Raven binary.
Building Raven now requires GTK4 development files discoverable by pkg-config.
GTK does not run inside the compositor process.
