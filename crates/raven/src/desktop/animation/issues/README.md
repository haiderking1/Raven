# Fullscreen follow-up

The user reports better smoothness with snapshot animations, but the original
right/bottom browser edge jump remains. They also report that some applications
take longer than others to enter fullscreen, unlike their Hyprland session.
The user identified Firefox as showing the edge flicker and then corrected the
assistant: Firefox does not have the slow fullscreen issue. Chromium has slow
fullscreen entry and apparently does not have the edge flicker. The assistant's
earlier claim that both browsers were slow was an unsupported inference. Keep
the Chromium flicker comparison tentative. The fullscreen trigger is still
unspecified. This is user-reported behavior, not an instrumented reproduction.

Fullscreen delay and edge flicker need separate investigation. The reported
browser differences do not establish either cause. Application identity helps locate a reproducer; it does
not justify an app-name exception, browser flags or relaxing buffer readiness.

## Confirmed implementation behavior

The renderer starts its 200 ms animation only after the matching serial applies
and the resize transaction releases displayed geometry. Client response and GPU
readiness therefore precede the full visual duration. The 300 ms transaction
deadline bounds coordination, not total fullscreen completion: an unready client
keeps its displayed allocation until a matching commit actually applies.
Unchanged siblings without a resize configure do not require a matching commit.
No measured evidence yet shows which wait dominates the reported slow apps.

The inspected Hyprland reference separates requested destination geometry from
applied client geometry, animating the destination without Raven's same
commit-gated start. It also conditionally projects or crops undersized content.
That is a real behavioral difference, not proof of the browser flicker cause.

Raven centers current window geometry in the fullscreen allocation. A later
geometry change can move that content origin. This is a candidate to investigate
against actual commits, not a confirmed defect or justification for globally
stretching every buffer.

No timeout, animation duration, synchronization or rendering policy was changed
in this follow-up. No tests, application launches or session changes occurred.
