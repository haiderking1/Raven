# Input configuration

Keyboard repeat updates use Smithay’s existing keyboard handle. Existing
wl_keyboard resources receive repeat_info and new bindings receive the stored
values. No keyboard replacement, synthetic releases, or shortcut ownership
changes occur during reload. Bootstrap-seat replacement uses configured values.

Mouse profiles use libinput configuration, not a substitution of unaccelerated
motion deltas. udev ID_INPUT_MOUSE selects mice; touchpads and pointing sticks
are excluded. Devices without configurable profiles are ignored. All connected
mice are preflighted before mutation. Setter failures restore previously changed
profiles before rejecting a candidate; rollback failures are included in the
report rather than hidden. Keyboard and other settings publish only afterward.

Device handles belong to the TTY backend. Removal forgets the device. Suspend
clears the registry after draining releases; resumed devices receive the current
profile through normal DeviceAdded handling. Teardown drops device handles before
the libinput context and libseat notifier. Hot-plug configuration failures keep
the new device’s existing profile and produce a log message.

Tests cover Lua and programmatic validation, preflight, rollback, restoring
per-device defaults, and repeat_info updates to existing and future clients.
Actual mouse response and session switching still require hardware testing.
