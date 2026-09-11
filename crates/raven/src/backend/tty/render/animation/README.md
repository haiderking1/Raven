# Resize group rendering

A transition retains an OLD GPU copy and the last accepted rendered group image.
During animation, the normal live surface-tree importer supplies current textures,
viewport source rectangles, buffer transforms, geometry offsets and subsurface
placement. Current content and borders render into a temporary full-opacity image
at the interpolated frame/client geometry. A GLES texture shader linearly combines
both premultiplied images. It does not fade each overlapping subsurface separately.

## Resources and synchronization

`Snapshot` owns its offscreen texture, copy fence and the exact input elements.
Those inputs retain Smithay Buffer references until the copy fence signals.
Renderer commands use one GLES context, so a later image read follows its copy
without a CPU wait. There are no pixel readbacks, fake attachments or manual
client-buffer releases.

Ordinary rendering retires completed input references. If cancellation drops a
copy before its fence completes, destruction waits for that fence before releasing
inputs. This is a resource-safety wait, not a client-relayout delay. A failed draw
also finishes and fences any partial work before dropping its inputs. The pinned
GLES frame finish implementation always supplies a fence or finishes synchronously.

Unique retained image storage is capped at 128 MiB across active transitions,
including retained interruption inputs and accepted current images. A pending copy
expires after two seconds if no applied displayed geometry becomes available.
Allocation, import and shader failures cancel the visual and use normal committed
rendering. No stationary-window framebuffer cache remains after transitions end.
The single compiled shader may remain for the lifetime of the Scene context.

`pause_frames` clears images and the program before EGL/DRM teardown. Renderer
context identity is checked as well as output identity, geometry, transform and
fractional scale. State contains settings and render-participation metadata only,
not GLES resources.

## Live accounting

A group has a compositor ID, never an old client ID. `accounting` records the live
surface IDs and regions from the actual current-image draws. It subtracts internal
opaque coverage, output clipping and opaque elements above the group. Old-image
surface IDs are never added. At progress zero, no live image participates.

`State::animation_render_states` expands only groups that the DRM render result
says were rendered. The three consumers are frame visibility, presentation feedback
and DMA-BUF feedback delivery. Every expanded live entry is `Rendering` with no
scanout reason. It cannot gain ZERO_COPY or a scanout preference merely because an
old snapshot still holds a client buffer. This is accounting for actual offscreen
composition, not a no-op surface proxy.

Each group advances its damage commit every sampled frame. New live commits are
therefore included even if progress rounds to unchanged pixel geometry. Group
removal damages its old extent and returns normal rendering to its usual IDs.
Opaque coverage of resampled images is rounded inward.

## Scheduling

An active animation requests one content redraw after each rendered frame.
Admission, pending/successor ownership and deadlines stay with the existing
scheduler. The resource-expiry deadline is one-shot; there is no animation tick
source or idle poll. An empty render cancels occluded running transitions instead
of repeatedly requesting no-damage frames. Known opaque scene coverage also
cancels a group before allocating its live offscreen image.

The existing GPU query still brackets submit rendering. Current-image offscreen
work happens during scene assembly before that query. The existing CPU render
measurement includes assembly. Extending GPU query scope needs a separate change
to the protected timing/submit boundary; this feature does not alter those files.

## Build and measurement limits

The locked/offline production check and release build passed after integration.
No tests or GPU execution checks ran, as requested. Live fullscreen behavior and
performance remain unverified with this build.

The existing GPU query starts in submission rendering. Snapshot copies and live
offscreen image construction occur outside that query. Per-frame scene assembly
is included in the existing CPU render duration; pre-commit snapshot capture is
outside that duration too. The GPU timing scope and scheduler were not changed,
and their current measurements must not be described as total animation GPU cost.
