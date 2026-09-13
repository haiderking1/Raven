# Window dragging

Hold Super and drag with the left mouse button. Super + right-drag resizes floating windows and tiled splits; see [resizing](resizing.md). Super + V toggles the focused window between tiled and floating. Fullscreen windows are excluded. Switching to floating restores the previous floating size around the current displayed tile center, not the screen center or a saved position. On the first toggle it uses the client geometry (640×400 if unavailable). Following Hyprland, when both size differences are less than 5 logical pixels, it adds 10 to each dimension to make the toggle visible. Client size constraints still apply. Returning to tiling restores its saved tile index where possible.

- Floating windows follow the pointer and may extend beyond workarea and output edges. Only automatic initial placement is constrained to the workarea. Their chosen position survives client commits, layout refresh, and fullscreen round trips.
- Tiled windows follow the pointer as a live render preview. Their layout membership stays unchanged during the drag. Releasing over another tiled window swaps the two layout positions through the existing resize transaction.
- There is no target highlight. Gaps, floating windows, panels, and the source tile are not drop targets.
- Releasing without a target or pressing Escape leaves tile order unchanged. Cancelling floating movement keeps its last position.
- Other compositor keyboard shortcuts cancel the drag before executing. Workspace/output changes, unmapping, fullscreen changes, and session suspension also cancel it.

Admission rejects fullscreen windows, existing pointer/keyboard grabs, pointer constraints, and top/overlay layer hits. Client button events are suppressed as complete press/release pairs, including releases arriving after cancellation. Floating movement and grab callbacks never reenter the public pointer handle while its mutex is held. Drop and cancellation refresh pointer focus outside that mutex.

Rendering lives in backend/tty/render/dragging. Preview elements preserve client surface identities and buffer ownership. Dragged windows do not start competing resize animations.

## Reference behavior

Reviewed Hyprland c31b90c5fc87b6bfc494f4e66d5acc3b0ba5b0ad, specifically layout/supplementary/DragController.cpp and layout/LayoutManager.cpp. Its current drag controller temporarily floats tiled targets and restores tiled membership on release. Raven deliberately implements the requested two-slot swap rather than copying layout-dependent reinsertion or group-drop behavior.

## QA

Build with cargo build -p raven and start ./target/debug/raven. Check floating movement and persistence, tiled swaps in both directions with unequal tile sizes, invalid drops, Escape, release after releasing Super first, and workspace/fullscreen/VT changes during dragging. No automated tests were added or run for this change.
