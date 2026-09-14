use super::native;
use crate::state::State;
impl State {
    pub(crate) fn refresh_screenshot(&mut self) {
        self.refresh_clipboard_transfers();
        if self.screenshot.active() {
            let valid = self
                .output
                .as_ref()
                .and_then(|o| self.space().output_geometry(o).map(|area| (o, area)))
                .is_some_and(|(o, area)| {
                    area == self.screenshot.area
                        && o.current_scale().fractional_scale() == self.screenshot.scale
                        && o.current_mode()
                            .map(|m| o.current_transform().transform_size(m.size))
                            == Some(self.screenshot.size)
                });
            if !valid
                || self.exclusive_keyboard_layer().is_some()
                || self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
                || self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
            {
                self.cancel_screenshot();
            }
        }
        if let Some((generation, image)) = self.backend.as_mut().and_then(|b| b.poll_screenshot()) {
            self.screenshot_pixels(generation, image);
        }
        let result = self.screenshot.encoding.as_ref().map(|r| r.try_recv());
        if let Some(result) = result {
            match result {
                Ok(done) => {
                    self.screenshot.encoding = None;
                    if let Some(png) = done.png {
                        smithay::wayland::selection::data_device::set_data_device_selection(
                            &self.display_handle,
                            &self.seat,
                            vec!["image/png".into()],
                            Some(png),
                        );
                    }
                    eprintln!("raven: {}", done.message);
                    self.screenshot.notice = done.hud.map(|p| native::hud_buffer(&p));
                    self.arm_screenshot_notice();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(_) => {
                    self.screenshot.encoding = None;
                    self.screenshot_notice("Screenshot encoder stopped before finishing");
                }
            }
        }
    }
}
