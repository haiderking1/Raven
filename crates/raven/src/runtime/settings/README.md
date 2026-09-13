# Runtime settings

Settings is the shared typed model for Lua configuration and run_with_settings.
It owns appearance, resize animation duration, keybindings, terminal/launcher
argv, startup entries, cursor theme/size, and workspace visibility choices.
Settings::validate checks values before application; cursor asset preparation
is also required before a Lua candidate is published.

Normal startup reads the [Lua configuration](../config/README.md) and starts its
watcher. Programmatic run_with_settings does not read or overwrite the user file.
Constructing settings never starts processes. Startup entries launch once after
the Wayland socket and backend are ready; they do not run again on reload.

The default startup entry remains Waybar. Lua can replace that list with any
commands, including an empty list. Raven never edits Waybar settings or chooses
a wallpaper tool for the user.

Appearance remains independently validated by State::set_appearance. Reloads
wait for current resize/fullscreen transactions and drags to settle before
applying a complete candidate. Invalid reloads preserve the active settings.

Frame scheduling, acquire fences, and presentation ownership are not part of
this user-facing settings model.
