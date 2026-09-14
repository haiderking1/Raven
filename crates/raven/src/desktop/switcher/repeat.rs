use crate::state::State;
use smithay::{
    backend::input::KeyState,
    input::keyboard::{Keycode, ModifiersState},
    reexports::calloop::{
        LoopHandle, RegistrationToken,
        timer::{TimeoutAction, Timer},
    },
};
use std::time::Duration;
#[derive(Default)]
pub(super) struct Repeat {
    handle: Option<LoopHandle<'static, State>>,
    token: Option<RegistrationToken>,
}
impl State {
    pub(crate) fn install_app_switcher(&mut self, handle: LoopHandle<'static, Self>) {
        self.switcher.repeat.handle = Some(handle);
    }
    pub(crate) fn switcher_key_state(
        &mut self,
        code: Keycode,
        state: KeyState,
        modifiers: &ModifiersState,
    ) -> bool {
        if self.switcher.repeat_follows_shift {
            self.switcher.reverse = modifiers.shift;
        }
        if state == KeyState::Released && self.switcher.repeat_key == Some(code) {
            self.stop_switcher_repeat();
        }
        if state == KeyState::Pressed {
            self.switcher.pending = None;
            self.switcher.exiting = None;
        }
        self.switcher.active() && !modifiers.alt
    }
    pub(crate) fn start_switcher_repeat(&mut self, code: Keycode, follows_shift: bool) {
        self.stop_switcher_repeat();
        if !self.switcher.active() || self.config.settings.input.keyboard.repeat_rate == 0 {
            return;
        }
        let Some(handle) = self.switcher.repeat.handle.clone() else {
            return;
        };
        self.switcher.repeat_follows_shift = follows_shift;
        let delay = self.config.settings.input.keyboard.repeat_delay as u64;
        let token = handle.insert_source(
            Timer::from_duration(Duration::from_millis(delay)),
            |_, _, state| {
                let rate = state.config.settings.input.keyboard.repeat_rate;
                if !state.switcher.active() || state.switcher.repeat_key.is_none() || rate == 0 {
                    state.switcher.repeat.token = None;
                    return TimeoutAction::Drop;
                }
                state.cycle_applications(state.switcher.reverse);
                TimeoutAction::ToDuration(Duration::from_secs_f64(1.0 / f64::from(rate)))
            },
        );
        match token {
            Ok(token) => {
                self.switcher.repeat.token = Some(token);
                self.switcher.repeat_key = Some(code);
            }
            Err(error) => eprintln!("raven: cannot repeat application switching: {error}"),
        }
    }
    pub(crate) fn stop_switcher_repeat(&mut self) {
        if let Some(token) = self.switcher.repeat.token.take() {
            if let Some(handle) = &self.switcher.repeat.handle {
                handle.remove(token);
            }
        }
        self.switcher.repeat_key = None;
        self.switcher.repeat_follows_shift = false;
    }
}
