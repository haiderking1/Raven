mod assertions;
mod fixture;
mod isolation;
mod protocol;

use super::{server::Server, x11::Client};
use crate::runtime::client::Clients;
pub(super) use fixture::Fixture;
use smithay::{
    desktop::{LayerSurface, Window},
    reexports::wayland_server::Resource,
};
use std::{error::Error, time::Instant};

pub(super) fn exercise(
    server: &mut Server,
    clients: &mut Clients,
    client: &mut Client,
    main: &Window,
    fixture: &Fixture,
    deadline: Instant,
) -> Result<LayerSurface, Box<dyn Error>> {
    fixture.start(clients);
    server.until(
        deadline,
        "native Waybar mapped, workspace protocol initialized and reserved tile settled",
        |server| {
            fixture.check_exit()?;
            client.poll()?;
            assertions::membership(server, main, 0)?;
            Ok(assertions::mapped(server).is_some()
                && protocol::initialized(&fixture.text()?)
                && main.geometry().size == (940, 488).into()
                && server.scene_size == 6)
        },
    )?;
    fixture.assert_running()?;
    let layer = assertions::mapped(server).ok_or("Waybar layer disappeared")?;
    assertions::reserved(server, main, &layer)?;
    let tile = server.state.window_tile_geometry(main);
    let frame = server.state.window_frame_geometry(main);
    eprintln!(
        "Waybar regression: native Top layer mapped, reservation=32px, scene=6; protocol log {}",
        fixture.log.display()
    );
    for (name, active, x) in [("2", 1, 72.0), ("1", 0, 24.0)] {
        let text = fixture.text()?;
        let handle = protocol::workspace(&text, name)?;
        let offset = text.len();
        server.click_layer(&layer, (x, 16.0).into())?;
        server.until(
            deadline,
            &format!("Waybar button {name} activate/commit, state/done and UI redraw"),
            |server| {
                fixture.check_exit()?;
                client.poll()?;
                assertions::membership(server, main, server.state.workspaces.active)?;
                Ok(server.state.workspaces.active == active
                    && server.scene_size == if active == 0 { 6 } else { 1 }
                    && protocol::activated(
                        &fixture.text()?[offset..],
                        &handle,
                        layer.wl_surface().id().protocol_id(),
                    ))
            },
        )?;
        assertions::membership(server, main, active)?;
        if server.state.window_tile_geometry(main) != tile
            || server.state.window_frame_geometry(main) != frame
        {
            return Err("Waybar workspace round trip changed original tile allocation".into());
        }
        eprintln!(
            "Waybar regression: real button {name} activated workspace {name}, acknowledged state and redrew UI"
        );
    }
    assertions::reserved(server, main, &layer)?;
    Ok(layer)
}

pub(super) fn fullscreen(
    server: &Server,
    main: &Window,
    layer: &LayerSurface,
    fixture: &Fixture,
) -> Result<(), Box<dyn Error>> {
    fixture.assert_running()?;
    assertions::fullscreen(server, main, layer)
}
