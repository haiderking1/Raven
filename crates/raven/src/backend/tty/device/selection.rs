use smithay::{
    backend::drm::DrmDevice,
    reexports::drm::control::{Device as _, Mode, ModeTypeFlags, connector, crtc},
};
use std::error::Error;

pub(super) struct Connector {
    pub info: connector::Info,
    pub crtcs: Vec<crtc::Handle>,
    pub modes: Vec<Mode>,
}

pub(super) fn connected(drm: &DrmDevice) -> Result<Vec<Connector>, Box<dyn Error>> {
    let resources = drm.resource_handles()?;
    let mut connectors = Vec::new();
    for handle in resources.connectors() {
        let info = match drm.get_connector(*handle, true) {
            Ok(info) => info,
            Err(error) => {
                eprintln!("raven: cannot inspect connector {handle:?}: {error}");
                continue;
            }
        };
        if info.state() != connector::State::Connected || info.modes().is_empty() {
            continue;
        }
        let mut crtcs = Vec::new();
        for encoder in info.encoders() {
            match drm.get_encoder(*encoder) {
                Ok(encoder) => {
                    for crtc in resources.filter_crtcs(encoder.possible_crtcs()) {
                        if !crtcs.contains(&crtc) {
                            crtcs.push(crtc);
                        }
                    }
                }
                Err(error) => eprintln!("raven: cannot inspect encoder {encoder:?}: {error}"),
            }
        }
        let mut modes = info.modes().to_vec();
        // Preserve the driver's order within each group, trying preferred modes first.
        modes.sort_by_key(|mode| !mode.mode_type().contains(ModeTypeFlags::PREFERRED));
        connectors.push(Connector { info, crtcs, modes });
    }
    Ok(connectors)
}
