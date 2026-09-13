# Layer surfaces

Launchers, panels, and wallpaper tools use wlr-layer-shell rather than XDG
toplevels. Raven advertises the protocol and assigns layer clients to its single
output, outside workspaces. See [wallpaper setup](wallpapers/README.md) for awww,
swaybg, and the generic startup path.

Only configured, buffer-backed surfaces enter the output layer map. Pending or
null-buffer-unmapped clients do not reserve space, receive frames, or take focus.
Mapped exclusive zones reduce the area available to tiles on every workspace.

Rendering and hit testing use Overlay, Top, windows, Bottom, Background order,
with the newest surface first within each layer. Popups and subsurfaces follow
their parent.

Exclusive Top/Overlay layers take keyboard focus without changing the remembered
window. OnDemand layers need a click; noninteractive layers never take it.
Closing or unmapping a launcher restores the active workspace's remembered
window. Workspace switches leave output layers visible.

Layout is refreshed on commits, membership changes, and output geometry changes,
not on every idle frame. Layer focus changes do not repeat keyboard enters or
window activation configures when the same launcher still owns focus.
