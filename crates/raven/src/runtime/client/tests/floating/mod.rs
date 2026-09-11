mod assertions;
mod tiles;

use super::{
    server::Server,
    x11::{Client, Command, FloatingKind},
};
use smithay::{desktop::Window, utils::IsAlive};
use std::{error::Error, time::Instant};
use tiles::Baseline;

pub(super) fn exercise(
    server: &mut Server,
    client: &mut Client,
    main: &Window,
    deadline: Instant,
) -> Result<(), Box<dyn Error>> {
    server.until(deadline, "initial normal main tile settles", |server| {
        client.poll()?;
        Ok(server.state.window_tile_geometry(main).is_some_and(|tile| {
            main.geometry().size == tile.size
                && main
                    .toplevel()
                    .is_some_and(|top| top.current_state().size == Some(tile.size))
        }))
    })?;
    let baseline = Baseline::capture(server, main)?;
    let transient = open(
        server,
        client,
        main,
        FloatingKind::Transient,
        &baseline,
        1,
        deadline,
    )?;
    let transient_area = assertions::floating(server, main, &transient, FloatingKind::Transient)?;
    assertions::hit(server, &transient, assertions::center(transient_area))?;
    let splash = open(
        server,
        client,
        main,
        FloatingKind::Splash,
        &baseline,
        2,
        deadline,
    )?;
    let splash_area = assertions::floating(server, main, &splash, FloatingKind::Splash)?;
    let transient_area = assertions::floating(server, main, &transient, FloatingKind::Transient)?;
    // Both windows overlap the main tile. The larger transient also has an
    // exposed corner outside the centered splash, so test both input targets.
    let corner = (transient_area.loc + smithay::utils::Point::from((1, 1))).to_f64();
    if splash_area.to_f64().contains(corner) {
        return Err("fixture leaves no exposed transient corner".into());
    }
    assertions::hit(server, &transient, corner)?;
    assertions::hit(server, &splash, assertions::center(splash_area))?;
    eprintln!(
        "satellite regression: transient and parentless splash floated at {transient_area:?} and {splash_area:?}; main tile unchanged, scene={}",
        server.scene_size
    );

    close(
        server,
        client,
        &splash,
        FloatingKind::Splash,
        &baseline,
        1,
        deadline,
    )?;
    assertions::floating(server, main, &transient, FloatingKind::Transient)?;
    assertions::hit(server, &transient, assertions::center(transient_area))?;
    close(
        server,
        client,
        &transient,
        FloatingKind::Transient,
        &baseline,
        0,
        deadline,
    )?;
    assertions::hit(server, main, assertions::center(transient_area))?;
    eprintln!(
        "satellite regression: both floating windows unmapped; original main tile and scene restored"
    );
    Ok(())
}

fn open(
    server: &mut Server,
    client: &mut Client,
    main: &Window,
    kind: FloatingKind,
    baseline: &Baseline,
    count: usize,
    deadline: Instant,
) -> Result<Window, Box<dyn Error>> {
    let before = server.state.windows.clone();
    client.command(Command::OpenFloating(kind))?;
    server.until(
        deadline,
        &format!("{kind:?} real map and scene import"),
        |server| {
            client.poll()?;
            Ok(server.state.windows.len() == count + 1
                && server.state.space().elements().count() == count + 1
                && server.state.windows.iter().all(assertions::imported)
                && server.scene_size == baseline.scene_size + 5 * count)
        },
    )?;
    let added: Vec<_> = server
        .state
        .windows
        .iter()
        .filter(|window| !before.contains(window))
        .cloned()
        .collect();
    let [window] = added.as_slice() else {
        return Err(format!("{kind:?} did not create exactly one new Raven toplevel").into());
    };
    assertions::floating(server, main, window, kind)?;
    baseline.check(server, count)?;
    client.command(Command::InspectFloating(kind))?;
    server.until(deadline, "X11 floating geometry reply", |_| {
        client.poll()?;
        Ok(client.floating_geometry[kind.index()].is_some())
    })?;
    let (width, height) = kind.size();
    if client.floating_geometry[kind.index()] != Some((width, height, true)) {
        return Err(format!(
            "{kind:?} X11 mapped geometry mismatch: {:?}, expected {width}x{height} viewable",
            client.floating_geometry[kind.index()]
        )
        .into());
    }
    baseline.check(server, count)?;
    Ok(window.clone())
}

fn close(
    server: &mut Server,
    client: &mut Client,
    window: &Window,
    kind: FloatingKind,
    baseline: &Baseline,
    remaining: usize,
    deadline: Instant,
) -> Result<(), Box<dyn Error>> {
    client.command(Command::CloseFloating(kind))?;
    server.until(
        deadline,
        &format!("{kind:?} unmap and scene removal"),
        |server| {
            client.poll()?;
            Ok(!window.alive()
                && !server.state.windows.contains(window)
                && server.state.windows.len() == remaining + 1
                && server.state.space().elements().count() == remaining + 1
                && server.state.space().element_location(window).is_none()
                && server.scene_size == baseline.scene_size + 5 * remaining)
        },
    )?;
    if server.state.window_is_floating(window) || server.state.floating_geometry(window).is_some() {
        return Err(format!("{kind:?} retained floating ownership after close").into());
    }
    baseline.check(server, remaining)
}
