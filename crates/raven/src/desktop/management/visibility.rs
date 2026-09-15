use super::{minimize::hidden, model::Minimized};
use crate::state::State;
use smithay::desktop::Window;
use std::sync::Mutex;
impl State {
    pub(crate) fn window_is_minimized(&self, window: &Window) -> bool {
        let mut current = Some(window);
        for _ in 0..=self.windows.len() {
            let Some(w) = current else {
                break;
            };
            if hidden(w) {
                return true;
            }
            current = self.valid_floating_parent(w);
        }
        false
    }
    pub(super) fn wants_minimized(&self, window: &Window) -> bool {
        let mut current = Some(window);
        for _ in 0..=self.windows.len() {
            let Some(w) = current else {
                break;
            };
            if w.user_data()
                .get::<Mutex<Minimized>>()
                .is_some_and(|v| v.lock().unwrap().requested)
            {
                return true;
            }
            current = self.valid_floating_parent(w);
        }
        false
    }
}
