# Coordination-held callbacks

Participant commits retain their coordination blockers and the 300 ms deadline. Smithay notifies Raven after caching a root transaction and its synchronized descendants, before application. Raven removes only committed frame callbacks. Desynchronized branches are excluded, and later child commits wait for another parent commit.

An independent calloop timer completes queued callbacks at the output refresh interval, with a 60 Hz fallback. Normal and background callback delivery share pacing state with this timer. Earlier current callbacks precede extracted requests. Later applied callbacks follow them. No redraw or page flip is required.

Callback completion does not apply visual state, clear GPU acquire blockers, release buffers, or claim presentation. Queues survive ordinary transaction release. Cancellation restores live requests to normal callback handling. Dead surfaces are discarded, and timer registration failure restores requests and releases only coordination.
