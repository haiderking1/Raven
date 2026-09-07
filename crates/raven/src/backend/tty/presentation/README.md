# Presentation feedback

Raven advertises wp_presentation with CLOCK_MONOTONIC. After rendering a changed frame, it collects feedback only for surfaces present in the renderer result, including mapped layers, popups, cursor surfaces, and drag icons. That feedback travels with the DRM frame, not with whatever surface state exists when the event arrives.

Only an accepted pageflip completes feedback. Monotonic kernel timestamps carry HW_CLOCK; missing, zero, or realtime timestamps use the current monotonic clock without that flag. Refresh comes from the selected output mode. Sequence numbers extend across 32-bit wraps; a backwards reset reports an unknown sequence instead of fabricating progress. Constant zero counters remain zero.

Feedback dropped by failed submission, session reset, or shutdown is discarded by Smithay. No-damage renders do not claim a new hardware presentation. Client feedback for such commits remains subject to later presentation or discard.
