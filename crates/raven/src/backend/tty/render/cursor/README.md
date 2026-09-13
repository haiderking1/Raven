# Themed cursors

Raven advertises wp_cursor_shape_manager_v1 version 2 through Smithay. Named
requests use SeatHandler::cursor_image after Smithay validates pointer focus and
the enter serial. All protocol shapes use their cursor-icon names and standard
aliases. No application-specific sizing rules or procedural arrow remain.

## Settings

Raven reads these at backend startup, before acquiring the seat:

- XCURSOR_THEME selects an installed Xcursor theme. The default is "default",
  which normally inherits the distribution's selected cursor theme.
- XCURSOR_SIZE selects a positive integer size in logical pixels. The default is 24.
- XCURSOR_PATH overrides theme search paths through the xcursor library.
  Otherwise its standard user and system icon directories are searched.

For example:

```sh
XCURSOR_THEME=Adwaita XCURSOR_SIZE=24 ./target/debug/raven
```

Theme and size changes currently require restarting Raven. Raven does not modify
GNOME settings, GTK files, Waybar settings, or client-provided cursor images.
SVG-only cursor themes are not supported by this raster Xcursor loader.

## Loading and fallback

Theme inheritance and standard aliases are resolved by the xcursor and
cursor-icon libraries. A local alias wins over an inherited canonical name.
The nearest nominal image size is selected for logical size times output scale,
with a larger image winning equal-distance ties. All animation frames at that
nominal size are retained. Full image bounds, alpha, and hotspots are preserved;
source pixels and logical destination dimensions are kept separate.

Each named shape and target pixel size is cached. A missing or malformed shape
logs once for that cache key and falls back to the selected theme's default
cursor. If no default cursor can be loaded, startup reports the error before
opening the seat rather than substituting a drawn arrow or invisible pointer.

## Animation and ownership

Frame delays come from the theme. Elapsed time selects the current frame, skipping
missed frames after delays. A zero frame delay is treated as one millisecond to
ensure progress. Single-frame cursors do not schedule animation wakes.

The existing backend one-shot timer wakes at the next cursor frame boundary.
An expired cursor deadline requests an ordinary redraw, subject to the existing
pending/successor frame admission. Consuming the deadline prevents a timer loop
while rendering waits for a pageflip. Static, hidden, and client-surface cursors
have no compositor animation timer. VT suspension cancels its deadline.

Memory cursor buffers remain eligible for the existing hardware cursor path and
software fallback. Client wl_pointer.set_cursor surfaces retain their own
buffer scale, viewport, hotspot, acquire fences, release, frame callback, and
presentation handling. DND icon rendering is unchanged. Smithay resets the
cursor when pointer focus is lost.

The tablet trait implementation only satisfies Smithay's shared cursor-shape
delegate bounds. Raven does not advertise tablet support.
