# Callback eligibility

Successful render results publish each active surface's actual visible pixel area. A visible surface gets at most one callback opportunity per submission/estimated-refresh cycle. An occluded entry with zero pixels is not treated as visible merely because it exists in the result map.

Mapped occluded surfaces with committed callbacks receive a fallback opportunity at most every 250 ms. A deadline exists only while a callback is pending; it does not request rendering. Hidden workspaces receive neither normal nor fallback callbacks. This keeps covered clients from running at output refresh without leaving them indefinitely stalled.

Ordinary visible scenes bypass the background traversal. Unknown render visibility remains conservatively eligible until the first result is available. Applied hidden-workspace commits still run protocol/lifecycle handling but do not request a repaint of the active output.
