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
pub fn surface_under(state: &State, location: Point<f64, Logical>)
    -> Option<(WlSurface, Point<f64, Logical>)>;
pub fn focus_window_at(state: &mut State, location: Point<f64, Logical>, serial: Serial);
```

`surface_under` returns the target surface and its origin in compositor logical
coordinates, including subsurfaces and popups. `focus_window_at` raises and gives
keyboard focus to the window at the supplied location. Input skips that call
while either the pointer or keyboard has a grab.

Map `state.output` into `state.space` with its current mode, scale, and transform.
Motion uses that logical output geometry and clamps to the last pixel. Without
a mapped output and valid mode, motion is ignored.

Dispatch only from the active libinput source. Backend session handling must
suspend that source while inactive. VT requests call `TtyBackend::change_vt`;
input does not manage session activation. Smithay resets the cursor through
`SeatHandler::cursor_image` when pointer focus leaves a client.

The four unit tests cover shortcut suppression, VT selection, coordinate bounds,
and scroll units/stops. No State, Cargo, main, or lib changes belong to this module.
