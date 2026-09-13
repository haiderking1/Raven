/// Keyboard repeat values sent to Wayland clients.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardSettings {
    pub repeat_rate: i32,
    pub repeat_delay: i32,
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            repeat_rate: 25,
            repeat_delay: 400,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseAccelProfile {
    #[default]
    Default,
    Flat,
    Adaptive,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputSettings {
    pub keyboard: KeyboardSettings,
    pub mouse_accel_profile: MouseAccelProfile,
}

impl InputSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !(0..=1000).contains(&self.keyboard.repeat_rate) {
            return Err(format!(
                "input.keyboard.repeat_rate: use a whole number from 0 to 1000 repeats per second. Received {}.",
                self.keyboard.repeat_rate
            ));
        }
        if !(0..=60000).contains(&self.keyboard.repeat_delay) {
            return Err(format!(
                "input.keyboard.repeat_delay: use a whole number from 0 to 60000 milliseconds. Received {}.",
                self.keyboard.repeat_delay
            ));
        }
        Ok(())
    }
}
