use crate::state::State;
use smithay::desktop::Window;

impl State {
    pub(crate) fn focused_window(&self) -> Option<Window> {
        let focus = self.seat.get_keyboard()?.current_focus()?;
        self.space()
            .elements()
            .filter(|window| self.window_is_visible(window))
            .find(|window| {
                let mut owns_focus = false;
                window.with_surfaces(|surface, _| owns_focus |= surface == &focus);
                owns_focus
            })
            .cloned()
    }
}
