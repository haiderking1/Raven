# Real Waybar phase

This helper belongs to the existing ignored satellite regression. It does not add
a test or run a synthetic workspace client. Run from the repository root:

```sh
timeout --signal=TERM --kill-after=20s 110s bash crates/raven/src/runtime/client/tests/server/run.sh
```

The shell entrypoint delegates to focused Python stdlib modules in
`server/runner/`. Cargo uses `--locked --offline` and a 90-second deadline.
Logs stay in the ignored `prefromance-inputlag/verification/` directory.

The wrapper creates a mode-0700 runtime with private HOME and XDG directories,
clears foreign display settings, and supplies environment changes only to its
children. Its foreground `dbus-daemon` listens at exactly
`unix:path=$XDG_RUNTIME_DIR/bus`. The generated config allows only the test UID
with EXTERNAL authentication. It has no service directories, includes,
activation helper, or systemd activation. A missing portal cannot be activated.
The system bus address also points to an absent socket inside the private runtime.
No ambient bus or global environment is used.

Teardown explicitly terminates and waits for the bus. The wrapper stops its
owned cargo process group on timeout or interruption and acts as a Linux child
subreaper for detached clients. On abort, its fallback checks `/proc` for the
exact runtime in both ownership environment variables, the same UID, and a
Waybar, Xwayland, or satellite executable. It rechecks identity across pidfd
creation and signals through pidfds, then reaps adopted children. Surviving
clients after a successful test make the wrapper fail before fallback cleanup.
The fixture records the live X11 socket and lock device, inode, UID, timestamps,
and lock owner inside the private runtime. After an abort, the wrapper removes
only unchanged recorded paths after the owner exits and the abstract listener
can be rebound. Logs identify forced cleanup and runtime removal. Other
sessions are not signalled.

`StartupPlan` launches installed Waybar with the generated config and CSS.
The phase requires a mapped 32px Top layer, production GLES import, unchanged
main tile membership, and workspace 2 then 1 clicks through the private seat.
Protocol logs must show activation, commit, state, done, and a new buffer
commit on the actual bar surface. Fullscreen must retain the owned Waybar
process and mapped layer while hiding the bar and removing gaps and borders.

## Earlier isolated result

After production began creating Smithay's XDG-output manager in State::new, the
complete existing integration passed with installed Waybar 0.15: real layer mapping and
GLES import, 32px reservation, workspace 2/1 button requests and UI redraw,
fullscreen hiding, gapless/borderless owner, and owned process/socket/file cleanup.
The successful run required no fallback process termination. The private bus's
missing portal is informational, not a compositor protocol failure.

This validated the isolated path, not real DRM presentation or live-session
performance. The helper never reads or rewrites personal Waybar configuration.

A later live report exposed a timing gap in this helper: it sends press and
release before dispatching Waybar's activation request, so it did not cover an
activation arriving during the held click. Raven now defers that panel request
until release. No tests were added or rerun for that correction, at the user's
request. The user subsequently confirmed that clicks worked after the Raven
correction and the requested personal configuration edit. See the
[issue history](../../../../protocols/workspace/bugs/README.md).
