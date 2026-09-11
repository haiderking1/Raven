# Panel click activation

The protocol transaction is accepted while the layer's implicit ClickGrab is
held, but the desktop switch waits for its release. No seat grab is removed or
bypassed. Window selections and custom grabs keep their existing rejection policy.

mod.rs owns the single committed intent and application. click.rs qualifies a
mapped layer click and checks the original grab identity and surface ancestry.
lifecycle.rs cancels weakly owned work at protocol and desktop lifetime boundaries.
The runtime's existing post-dispatch refresh processes pending work; an idle
compositor gets no new timer or wakeup.

The earlier isolated Waybar helper sent press and release before dispatching the
client request, so that check did not cover a request arriving while held. No
tests were added or run for this fix, as requested. Production compilation and
the release build passed. After the Raven correction and an explicitly requested
Waybar on-click configuration edit, the user confirmed that workspace clicks
worked. This does not validate every cancellation path or measure latency.
See the [issue history](../bugs/README.md) for the separate causes and evidence.
