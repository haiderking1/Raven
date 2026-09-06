# TTY backend integration

Call before dispatching Wayland clients:


    TtyBackend::install(&mut state, event_loop.handle())?;

The backend installs DRM, libinput, udev, libseat notifier, and timer sources.
It sets State.backend, State.output, and maps the output at logical 0,0.
No runtime render hook is required. State.refresh and State.send_frames remain
owned by the desktop implementation.

Public methods:

- install takes a mutable State and LoopHandle<'static, State>.
- change_vt takes an i32 and asks libseat to switch VTs.
- seat_name returns the actual libseat seat name as a String.
- failure returns Option<&str> for a fatal runtime error.

After the event loop exits, main should inspect failure, log it, and return an
error for a nonzero exit. Drop State.backend before dropping the event loop.
Backend Drop removes device sources, releases DRM and input devices, then removes
the session notifier. It also removes the wl_output global. Do not remove its
sources from outside the backend.

Install replaces the bootstrap wl_seat if its name differs from libseat's name.
It removes the old global and capabilities first. Call install before clients
can bind the bootstrap seat. Input events go to crate::input::handle_event.
On seat pause, the backend drains libinput's release events through that handler
to clear held keys, shortcut dispositions, and pointer grabs. It discards queued
presses and motion, then renders nothing until activation completes.

# Rendering and recovery

The backend probes every seat GPU until a connected connector, compatible CRTC,
mode, and GBM/GLES format combination succeeds. Preferred modes are tried first.
DrmCompositor renders the Space, including XDG popups, plus client cursors with
hotspots and a DND surface. Named cursors use a procedural arrow.

Only one frame is in flight. Pageflips pace changing content. A refresh-rate timer
checks damage and sends callbacks when unchanged content produces no pageflip.
A three-second pending-flip timeout stops the loop rather than reusing a buffer
still owned by KMS.

Resume resets the DRM device, clears compositor pending and queued frames,
invalidates buffer ages, and drains old pageflips before scheduling a fresh frame.
Selected-device removal, selected-output disconnection, and unrecoverable DRM
errors stop the loop. Connector changes while inactive are checked on resume.

# Current limits

- One GPU and one output. No live migration, multi-output layout, or mode switching.
- GBM/GLES composition only. No client direct scanout, overlay planes, or hardware cursor.
- Every named cursor shape uses the arrow. Client cursor surfaces are supported.
- DND icons are drawn at the pointer; this backend does not accumulate client
  wl_surface.attach offsets for them.
- Startup requires an already active libseat session and fails with instructions
  otherwise. Use a login user's active VT with logind or seatd access, not root.
- Output make/model remain generic. This backend does not advertise DMA-BUF,
  presentation-time, or explicit-sync protocols on its own.

The two scheduler regression tests passed using their standalone Rust test
harness. Full cargo check awaits main's module integration. KMS modesetting,
VT switching, and hot-unplug still need a hardware run.
