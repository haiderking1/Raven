use super::Server;
use smithay::{
    backend::input::ButtonState,
    desktop::LayerSurface,
    input::pointer::{ButtonEvent, MotionEvent},
    utils::{Logical, Point, SERIAL_COUNTER},
};
use std::error::Error;

impl Server {
    /// Send through the private seat only, never a host input device or socket.
    pub fn click_layer(
        &mut self,
        layer: &LayerSurface,
        location: Point<f64, Logical>,
    ) -> Result<(), Box<dyn Error>> {
        let focus = self
            .state
            .surface_under(location)
            .ok_or("no surface at Waybar button")?;
        if &focus.0 != layer.wl_surface() {
            return Err("workspace button does not hit real Waybar".into());
        }
        let pointer = self
            .state
            .seat
            .get_pointer()
            .ok_or("private seat has no pointer")?;
        self.state.pointer_location = location;
        let time = self.state.start_time.elapsed().as_millis() as u32;
        pointer.motion(
            &mut self.state,
            Some(focus),
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        pointer.frame(&mut self.state);
        for state in [ButtonState::Pressed, ButtonState::Released] {
            pointer.button(
                &mut self.state,
                &ButtonEvent {
                    button: 0x110,
                    state,
                    serial: SERIAL_COUNTER.next_serial(),
                    time,
                },
            );
            pointer.frame(&mut self.state);
        }
        self.state.display_handle.flush_clients()?;
        Ok(())
    }
}
