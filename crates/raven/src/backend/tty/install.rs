use super::{TtyBackend, device::Device, render::Scene, schedule::Schedule, sources::Sources};
use crate::state::State;
use smithay::{
    backend::{
        libinput::LibinputSessionInterface,
        session::{Session, libseat::LibSeatSession},
        udev::UdevBackend,
    },
    reexports::{calloop::LoopHandle, input::Libinput},
};
use std::{error::Error, time::Instant};

pub(super) fn install(
    state: &mut State,
    handle: LoopHandle<'static, State>,
) -> Result<(), Box<dyn Error>> {
    if state.backend.is_some() || state.output.is_some() {
        return Err(
            "the TTY backend must be installed exactly once before dispatching clients".into(),
        );
    }
    let (mut session, notifier) = LibSeatSession::new()
        .map_err(|error| format!("cannot open a libseat session: {error}; run Raven as your login user on an active VT with logind or seatd access"))?;
    if !session.is_active() {
        return Err(format!(
            "libseat seat '{}' is inactive; switch to the launching VT and start Raven again",
            session.seat()
        )
        .into());
    }
    let udev = UdevBackend::new(session.seat())?;
    let (device, drm_notifier) = Device::probe(&mut session, &udev)?;
    let mut input =
        Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(session.clone().into());
    input.udev_assign_seat(&session.seat()).map_err(|_| {
        format!(
            "libinput could not assign libseat seat '{}'",
            session.seat()
        )
    })?;
    let refresh = device
        .output
        .current_mode()
        .ok_or("selected output has no mode")?
        .refresh;
    let mut backend = TtyBackend {
        device: Some(device),
        session,
        input: Some(input),
        scene: Scene::new(),
        schedule: Schedule::new(refresh, Instant::now()),
        sources: Sources::new(handle),
        display_handle: state.display_handle.clone(),
        output_global: None,
        failure: None,
    };
    backend.sources.attach(
        notifier,
        drm_notifier,
        udev,
        backend.input.as_ref().expect("input initialized").clone(),
        backend.schedule.interval(),
    )?;
    replace_bootstrap_seat(state, &backend.session.seat())?;
    let output = backend
        .device
        .as_ref()
        .expect("device initialized")
        .output
        .clone();
    backend.output_global = Some(output.create_global::<State>(&state.display_handle));
    state.space.map_output(&output, (0, 0));
    state.output = Some(output);
    state.backend = Some(backend);
    // The already-registered timer performs the first render after install returns.
    Ok(())
}

fn replace_bootstrap_seat(state: &mut State, name: &str) -> Result<(), Box<dyn Error>> {
    if state.seat.name() == name {
        return Ok(());
    }
    let mut seat = state.seat_state.new_wl_seat(&state.display_handle, name);
    if let Err(error) = seat.add_keyboard(Default::default(), 400, 25) {
        if let Some(global) = seat.global() {
            state.display_handle.remove_global::<State>(global);
        }
        return Err(error.into());
    }
    seat.add_pointer();
    state.seat.remove_keyboard();
    state.seat.remove_pointer();
    if let Some(global) = state.seat.global() {
        state.display_handle.remove_global::<State>(global);
    }
    state.seat = seat;
    Ok(())
}
