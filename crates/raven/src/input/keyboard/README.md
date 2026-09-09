# Keyboard controls

- Super+Q launches Foot.
- Super+D launches Fuzzel (the `fuzzel` executable must be on PATH).
- Super+C asks the focused window to close.
- Super+F toggles fullscreen for the focused window. Extra Shift, Ctrl, or Alt disables this binding.
- Super+Shift+Q exits Raven.
- Ctrl+Alt+F1 through F12 switches VT.
- Super+1 through 9 selects workspace 1 through 9.
- Super+0 selects workspace 10.
- Super+Shift+number moves the focused window to that workspace without following.

Workspace bindings ignore extra Ctrl or Alt modifiers. Shortcut presses and
releases stay out of clients, and holding a key does not repeat its action.

Fullscreen uses the full output, including space reserved by panels. Exiting
restores the current tile or floating allocation without changing tile order. Each workspace has one
fullscreen owner; switching workspaces does not cancel fullscreen.

Each workspace remembers its tile order and keyboard focus. Switching dismisses
open menus. Workspace changes are ignored during a selection or drag-and-drop
grab, until the grab ends, so another client cannot receive an unmatched release.
