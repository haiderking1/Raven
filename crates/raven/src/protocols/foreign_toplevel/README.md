# Foreign toplevel management

This module implements wlr-foreign-toplevel-management-unstable-v1 versions 1–3
for trusted session taskbars and docks. It does not identify applications through
allowlists or grant access to private diagnostic windows.

manager.rs owns subscriptions and stop semantics. publish.rs sends metadata,
output association, parent relationships, state, and closed notifications.
requests.rs routes actions through desktop window-management operations.
capabilities.rs keeps XDG minimize availability aligned with restore handles.
fullscreen.rs queues a user-requested fullscreen transition behind focus changes.
click.rs defers taskbar activation until an implicit button grab releases; other
grabs reject it. Source-handle destruction, workspace changes, keyboard actions,
and session suspension cancel queued clicks. No delay or polling timer is added.

Subscriptions are capped at 32. Each subscription retains at most one entry per
mapped window, including client-destroyed handles until that window unmaps. This
prevents repeated advertisements after a client relinquishes a handle. Existing
handles continue working after manager stop. Closed handles are marked inactive
before notification, so later requests cannot target a remapped window.

Taskbar output events describe the assigned output, including minimized windows
and windows on another workspace, so the single-output taskbar can restore them.
Positive set_rectangle hints are accepted but no minimize animation uses them yet;
negative extents produce the specified invalid_rectangle protocol error.

Protocol events do not request compositor redraws. Window operations retain their
existing redraw and transaction paths. GPU resources are not owned by this module.
