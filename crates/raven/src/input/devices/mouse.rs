use super::transaction::Mouse;
use smithay::reexports::input::{AccelProfile, Device};

pub(super) fn configurable_mouse(device: &Device) -> bool {
    // Raven creates these devices through libinput's udev backend. The returned
    // udev handle retains its originating context; no foreign context is supplied.
    let Some(udev) = (unsafe { device.udev_device() }) else {
        return false;
    };
    let is = |name| udev.property_value(name).is_some_and(|value| value == "1");
    is("ID_INPUT_MOUSE")
        && !is("ID_INPUT_TOUCHPAD")
        && !is("ID_INPUT_POINTINGSTICK")
        && !device.config_accel_profiles().is_empty()
}

impl Mouse for Device {
    fn name(&self) -> String {
        Device::name(self).to_owned()
    }
    fn current(&self) -> Option<AccelProfile> {
        self.config_accel_profile()
    }
    fn default_profile(&self) -> Option<AccelProfile> {
        self.config_accel_default_profile()
    }
    fn supports(&self, profile: AccelProfile) -> bool {
        self.config_accel_profiles().contains(&profile)
    }
    fn set(&mut self, profile: AccelProfile) -> Result<(), String> {
        self.config_accel_set_profile(profile)
            .map_err(|error| format!("{error:?}"))
    }
}
