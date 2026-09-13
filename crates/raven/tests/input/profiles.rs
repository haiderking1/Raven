use crate::{
    runtime::settings::{InputSettings, MouseAccelProfile},
    transaction::{self, Mouse},
};
use smithay::reexports::input::AccelProfile::{self, Adaptive, Flat};

struct Device {
    current: AccelProfile,
    default: AccelProfile,
    flat_supported: bool,
    fail_flat: bool,
    writes: Vec<AccelProfile>,
}
impl Default for Device {
    fn default() -> Self {
        Self {
            current: Adaptive,
            default: Adaptive,
            flat_supported: true,
            fail_flat: false,
            writes: Vec::new(),
        }
    }
}
impl Mouse for Device {
    fn name(&self) -> String {
        "test mouse".into()
    }
    fn current(&self) -> Option<AccelProfile> {
        Some(self.current)
    }
    fn default_profile(&self) -> Option<AccelProfile> {
        Some(self.default)
    }
    fn supports(&self, profile: AccelProfile) -> bool {
        profile != Flat || self.flat_supported
    }
    fn set(&mut self, profile: AccelProfile) -> Result<(), String> {
        if self.fail_flat && profile == Flat {
            return Err("device refused profile".into());
        }
        self.writes.push(profile);
        self.current = profile;
        Ok(())
    }
}

#[test]
fn unsupported_profile_rejects_before_mutating_any_device() {
    let mut devices = [
        Device::default(),
        Device {
            flat_supported: false,
            ..Default::default()
        },
    ];
    assert!(transaction::apply(&mut devices, MouseAccelProfile::Flat).is_err());
    assert!(
        devices
            .iter()
            .all(|d| d.writes.is_empty() && d.current == Adaptive)
    );
}

#[test]
fn failed_device_update_restores_earlier_devices() {
    let mut devices = [
        Device::default(),
        Device {
            fail_flat: true,
            ..Default::default()
        },
    ];
    assert!(transaction::apply(&mut devices, MouseAccelProfile::Flat).is_err());
    assert_eq!(devices[0].writes, [Flat, Adaptive]);
    assert!(devices.iter().all(|d| d.current == Adaptive));
}

#[test]
fn default_restores_each_device_and_unchanged_profiles_are_not_written() {
    let mut devices = [
        Device::default(),
        Device {
            default: Flat,
            ..Default::default()
        },
    ];
    transaction::apply(&mut devices, MouseAccelProfile::Flat).unwrap();
    transaction::apply(&mut devices, MouseAccelProfile::Flat).unwrap();
    assert!(devices.iter().all(|d| d.writes == [Flat]));
    transaction::apply(&mut devices, MouseAccelProfile::Default).unwrap();
    assert_eq!(devices[0].current, Adaptive);
    assert_eq!(devices[1].current, Flat);
}

#[test]
fn programmatic_settings_validate_repeat_values() {
    let mut settings = InputSettings::default();
    assert_eq!(
        (
            settings.keyboard.repeat_rate,
            settings.keyboard.repeat_delay
        ),
        (25, 400)
    );
    settings.keyboard.repeat_rate = -1;
    assert!(settings.validate().is_err());
    settings.keyboard.repeat_rate = 0;
    settings.keyboard.repeat_delay = -1;
    assert!(settings.validate().is_err());
    settings.keyboard.repeat_delay = 0;
    assert!(settings.validate().is_ok());
}
