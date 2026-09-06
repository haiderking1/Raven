use super::events;
use crate::state::State;
use smithay::{
    backend::{
        drm::DrmDeviceNotifier,
        libinput::LibinputInputBackend,
        session::{Session, libseat::LibSeatSessionNotifier},
        udev::UdevBackend,
    },
    reexports::{
        calloop::{
            LoopHandle, RegistrationToken,
            timer::{TimeoutAction, Timer},
        },
        input::Libinput,
    },
};
use std::{error::Error, time::Duration};

/// Own registration tokens even during a partially failed install.
pub(super) struct Sources {
    handle: LoopHandle<'static, State>,
    devices: Vec<RegistrationToken>,
    session: Option<RegistrationToken>,
    unregistered_session: Option<LibSeatSessionNotifier>,
}

impl Sources {
    pub fn new(handle: LoopHandle<'static, State>) -> Self {
        Self {
            handle,
            devices: Vec::new(),
            session: None,
            unregistered_session: None,
        }
    }

    pub fn attach(
        &mut self,
        notifier: LibSeatSessionNotifier,
        drm: DrmDeviceNotifier,
        udev: UdevBackend,
        input: Libinput,
        interval: Duration,
    ) -> Result<(), Box<dyn Error>> {
        match self
            .handle
            .insert_source(notifier, |event, _, state| events::session(event, state))
        {
            Ok(token) => self.session = Some(token),
            Err(error) => {
                // Its strong libseat reference must outlive device teardown even
                // when registering the session notifier itself fails.
                self.unregistered_session = Some(error.inserted);
                return Err(error.error.into());
            }
        }
        self.devices.push(
            self.handle
                .insert_source(drm, |event, _, state| events::drm(event, state))
                .map_err(|error| error.error)?,
        );
        self.devices.push(
            self.handle
                .insert_source(udev, |event, _, state| events::udev(event, state))
                .map_err(|error| error.error)?,
        );
        self.devices.push(
            self.handle
                .insert_source(LibinputInputBackend::new(input), |event, _, state| {
                    if state.backend.as_ref().is_some_and(|backend| {
                        backend.failure.is_none()
                            && backend.schedule.active()
                            && backend.session.is_active()
                    }) {
                        crate::input::handle_event(event, state);
                    }
                })
                .map_err(|error| error.error)?,
        );
        self.devices.push(
            self.handle
                .insert_source(Timer::immediate(), move |_, _, state| {
                    events::timer(state);
                    TimeoutAction::ToDuration(interval)
                })
                .map_err(|error| error.error)?,
        );
        Ok(())
    }

    pub fn remove_devices(&mut self) {
        for token in self.devices.drain(..).rev() {
            self.handle.remove(token);
        }
    }

    pub fn remove_session(&mut self) {
        if let Some(token) = self.session.take() {
            self.handle.remove(token);
        }
        self.unregistered_session.take();
    }
}

impl Drop for Sources {
    fn drop(&mut self) {
        self.remove_devices();
        self.remove_session();
    }
}
