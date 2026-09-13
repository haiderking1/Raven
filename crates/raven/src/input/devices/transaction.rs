use crate::runtime::settings::MouseAccelProfile;
use smithay::reexports::input::AccelProfile;

pub(super) trait Mouse {
    fn name(&self) -> String;
    fn current(&self) -> Option<AccelProfile>;
    fn default_profile(&self) -> Option<AccelProfile>;
    fn supports(&self, profile: AccelProfile) -> bool;
    fn set(&mut self, profile: AccelProfile) -> Result<(), String>;
}

/// Preflight every device before mutation, then restore changed devices on failure.
pub(super) fn apply<D: Mouse>(devices: &mut [D], profile: MouseAccelProfile) -> Result<(), String> {
    let mut changes = Vec::new();
    for (index, device) in devices.iter().enumerate() {
        let target = match profile {
            MouseAccelProfile::Default => device.default_profile(),
            MouseAccelProfile::Flat => Some(AccelProfile::Flat),
            MouseAccelProfile::Adaptive => Some(AccelProfile::Adaptive),
        };
        let Some(target) = target else { continue };
        if !device.supports(target) {
            return Err(format!(
                "input.mouse.accel_profile: mouse {:?} does not support {target:?}. Choose 'default' or another supported profile.",
                device.name()
            ));
        }
        let previous = device
            .current()
            .ok_or_else(|| format!("Cannot read acceleration profile for {:?}.", device.name()))?;
        if previous != target {
            changes.push((index, previous, target));
        }
    }
    for (position, &(index, _, target)) in changes.iter().enumerate() {
        if let Err(error) = devices[index].set(target) {
            let mut report = format!(
                "input.mouse.accel_profile: cannot configure {:?}: {error}",
                devices[index].name()
            );
            for &(changed, previous, _) in changes[..position].iter().rev() {
                if let Err(error) = devices[changed].set(previous) {
                    report.push_str(&format!(
                        "\nCould not restore {:?}: {error}",
                        devices[changed].name()
                    ));
                }
            }
            return Err(report);
        }
    }
    Ok(())
}
