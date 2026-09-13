mod keyboard;
mod mouse;
mod transaction;

use crate::{
    runtime::settings::{InputSettings, MouseAccelProfile},
    state::State,
};
use smithay::reexports::input::Device;

/// Owned by the backend so device handles are released before libseat teardown.
#[derive(Debug, Default)]
pub(crate) struct Devices {
    mice: Vec<Device>,
}

impl Devices {
    pub(crate) fn clear(&mut self) {
        self.mice.clear();
    }

    fn added(&mut self, mut device: Device, profile: MouseAccelProfile) {
        if !mouse::configurable_mouse(&device) || self.mice.contains(&device) {
            return;
        }
        if let Err(error) = transaction::apply(std::slice::from_mut(&mut device), profile) {
            eprintln!("raven: mouse connected with its existing profile: {error}");
        }
        self.mice.push(device);
    }
}

impl State {
    pub(crate) fn set_input_settings(&mut self, settings: InputSettings) -> Result<(), String> {
        settings.validate()?;
        if let Some(backend) = &mut self.backend {
            transaction::apply(
                &mut backend.input_devices.mice,
                settings.mouse_accel_profile,
            )?;
        }
        keyboard::apply(&self.seat, settings.keyboard);
        Ok(())
    }

    pub(super) fn input_device_added(&mut self, device: Device) {
        if let Some(backend) = &mut self.backend {
            backend
                .input_devices
                .added(device, self.config.settings.input.mouse_accel_profile);
        }
    }

    pub(super) fn input_device_removed(&mut self, device: &Device) {
        if let Some(backend) = &mut self.backend {
            backend.input_devices.mice.retain(|mouse| mouse != device);
        }
    }
}
