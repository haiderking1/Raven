use crate::state::State;
use smithay::{
    backend::input::ButtonState, reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::Serial,
};

/// Keep the most recent delivered click after release. Tray menus may need a
/// D-Bus round trip before GTK creates their popup and requests its grab.
#[derive(Debug)]
pub(in crate::input) struct PopupClick {
    focus: WlSurface,
    button: u32,
    press: Serial,
    release: Option<Serial>,
}

impl State {
    pub(super) fn record_popup_click(&mut self, button: u32, state: ButtonState, serial: Serial) {
        match state {
            ButtonState::Pressed => {
                self.input.popup_click = self
                    .seat
                    .get_pointer()
                    .and_then(|pointer| pointer.current_focus())
                    .map(|focus| PopupClick {
                        focus,
                        button,
                        press: serial,
                        release: None,
                    });
            }
            ButtonState::Released => {
                if let Some(click) = &mut self.input.popup_click
                    && click.button == button
                {
                    click.release = Some(serial);
                }
            }
        }
    }

    pub(crate) fn popup_click_focus(&self, serial: Serial) -> Option<&WlSurface> {
        self.input
            .popup_click
            .as_ref()
            .filter(|click| click.press == serial || click.release == Some(serial))
            .map(|click| &click.focus)
    }

    pub(crate) fn clear_popup_click(&mut self) {
        self.input.popup_click = None;
    }
}
