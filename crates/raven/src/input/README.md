# Input integration

Entry point:

```rust
pub fn handle_event(event: InputEvent<LibinputInputBackend>, state: &mut State);
```

Add `pub input: crate::input::InputState` to `State` and initialize it with
`Default::default()`. Keep it across events so intercepted keys retain their
press/release disposition.

The desktop calls use these contracts:

```rust
impl State {
    pub fn surface_under(&self, location: Point<f64, Logical>)
        -> Option<(WlSurface, Point<f64, Logical>)>;
    pub fn focus_window_at(&mut self, location: Point<f64, Logical>);
}
```

`surface_under` returns the target surface and its origin in compositor logical
coordinates, including subsurfaces and popups. `focus_window_at` raises and gives
keyboard focus to the window at the supplied location. Input skips that call
while either the pointer or keyboard has a grab.

Map `state.output` into the active workspace through `state.space_mut()` with its
current mode, scale, and transform.
Motion uses that logical output geometry and clamps to the last pixel. Without
a mapped output and valid mode, motion is ignored.

Dispatch only from the active libinput source. Backend session handling must
suspend that source while inactive. VT requests call `TtyBackend::change_vt`;
input does not manage session activation. Smithay resets the cursor through
`SeatHandler::cursor_image` when pointer focus leaves a client.

Keyboard bindings are listed in [keyboard/README.md](keyboard/README.md).
Focused tests cover shortcut suppression and matching, coordinate bounds, and
scroll units/stops.
