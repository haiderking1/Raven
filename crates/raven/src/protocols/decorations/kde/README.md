# KDE server decorations

Raven advertises org_kde_kwin_server_decoration_manager version 1 with the
Server default mode. New decoration objects receive Server immediately. Every
requested mode receives Server, with at most four request replies per object
to stop clients that echo mode events from creating a feedback loop. This
matches Hyprland’s server-decoration policy.

GTK uses this manager’s default mode when deciding whether to create ordinary
client decorations. Ghostty also checks for the manager. XDG decoration
negotiation remains independent and unchanged. App-owned header bars and
explicit client-side preferences inside an application can still draw UI.

The request counter belongs to the Wayland decoration resource, not the
window or client. Release and disconnect drop it automatically. A replacement
object starts a fresh counter. Dedicated dispatch supplies this bounded
policy without changing Smithay’s default request-echoing handler.

Protocol tests use these production handlers with a minimal compositor:

```sh
cargo test --locked -p raven --test decorations
```

They check global and per-surface announcements, conflicting and unknown
preferences, reply bounds, independent objects, recreation, and destruction
order. They do not verify application rendering in a live Raven session.
