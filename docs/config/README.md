# Offline configuration handbook

Open `docs/config/index.html` in a browser. No server, network requests, external
fonts, or JavaScript framework are required. The pages remain readable and
navigable with JavaScript disabled. Copy buttons and section filtering are
optional enhancements; when clipboard access is blocked, the example is selected
for manual copying.

## Maintaining the handbook

Each topic has its own JSON file under `content/`. The section order lives in
`content/sections.json`. Shared HTML, styles, and browser behavior live in
`templates/` and `assets/`. The generated HTML is included for direct opening.

From the repository root:

```sh
python3 docs/config/build.py
python3 docs/config/checks/verify.py
# Also check every Lua example with the actual configuration loader:
python3 docs/config/checks/verify.py --raven target/debug/raven
```

The last command needs the terminal/launcher executables used in the examples
and a usable cursor theme installed. It does not execute the examples' startup
commands or launch a compositor. It uses temporary files, never the user config.

Keep examples consistent with `crates/raven/src/runtime/config/reference.md` and
the actual Lua validators. Mark unsupported behavior explicitly instead of
presenting planned features as current settings.

## Reference style

Organize pages for lookup: a short description, a working example, an option
table where useful, and links to related sections. Reserve troubleshooting for
specific symptoms and fixes. Keep setup lessons and implementation details out
of the user reference.

The organization was informed by [ArchWiki style guidance](https://wiki.archlinux.org/title/Help:Style)
and Hyprland's [variables](https://wiki.hypr.land/Configuring/Variables/) and
[bindings](https://wiki.hypr.land/Configuring/Binds/) references. Examples and
option definitions describe Raven, not Hyprland syntax or behavior.
