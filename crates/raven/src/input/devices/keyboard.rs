use crate::runtime::settings::KeyboardSettings;
use smithay::input::{Seat, SeatHandler};

pub(crate) fn apply<D: SeatHandler + 'static>(seat: &Seat<D>, settings: KeyboardSettings) {
    if let Some(keyboard) = seat.get_keyboard() {
        keyboard.change_repeat_info(settings.repeat_rate, settings.repeat_delay);
    }
}
