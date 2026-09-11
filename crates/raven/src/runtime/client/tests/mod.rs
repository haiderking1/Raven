mod appearance;
mod floating;
mod server;
mod waybar;
mod x11;

use super::Clients;
use server::{Server, sockets::Sockets};
use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use x11::{Client, Command};

#[test]
#[ignore = "requires installed satellite, Xwayland, Waybar and EGL; run tests/server/run.sh"]
fn installed_satellite_maps_and_fullscreens_through_real_scene() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(std::io::stderr)
        .try_init();
    let fixture = waybar::Fixture::new()?;
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut server = Server::new()?;
    let mut clients = Clients::new(server.socket.clone().into_os_string());
    // Descendant access deliberately verifies the production reservation, not ambient DISPLAY.
    let display = clients
        .satellite
        .as_ref()
        .ok_or("production Clients failed to initialize satellite")?
        .display()
        .ok_or("satellite reservation is unavailable")?
        .to_owned();
    let display = display.to_str().ok_or("reserved DISPLAY is not UTF-8")?;
    let sockets = Sockets::owned(display, &server.socket)?;
    eprintln!(
        "satellite regression: reserved {display}, Wayland {}",
        server.socket.display()
    );
    let mut client = Client::start(display, deadline)?;
    let result = (|| -> Result<(), Box<dyn Error>> {
        server.until(
            deadline,
            "X11 window buffer committed, mapped and imported",
            |server| {
                client.poll()?;
                Ok(client.mapped
                    && server.state.windows.len() == 1
                    && server.state.space().elements().count() == 1
                    && server.scene_size > 0
                    && server.state.windows[0].toplevel().is_some_and(|top| {
                        with_renderer_surface_state(top.wl_surface(), |state| {
                            state.buffer().is_some()
                        })
                        .unwrap_or(false)
                    }))
            },
        )?;
        let window = server.state.windows[0].clone();
        if server.state.fullscreen_window().is_some() {
            return Err("window became fullscreen before the X11 request".into());
        }
        eprintln!(
            "satellite regression: real Raven mapping and scene import observed, elements={}",
            server.scene_size
        );
        floating::exercise(&mut server, &mut client, &window, deadline)?;
        let bar = waybar::exercise(
            &mut server,
            &mut clients,
            &mut client,
            &window,
            &fixture,
            deadline,
        )?;
        client.command(Command::Fullscreen)?;
        server.until(
            deadline,
            "committed fullscreen state and output-sized window",
            |server| {
                client.poll()?;
                let current = window
                    .toplevel()
                    .ok_or("X11 toplevel disappeared")?
                    .current_state();
                Ok(server.state.fullscreen_window() == Some(&window)
                    && current.states.contains(xdg_toplevel::State::Fullscreen)
                    && current.size == Some(server.area.size)
                    && window.geometry().size == server.area.size
                    && server.state.window_layout_geometry(&window) == Some(server.area)
                    && server.state.space().element_location(&window) == Some(server.area.loc)
                    && server.scene_size > 0)
            },
        )?;
        waybar::fullscreen(&server, &window, &bar, &fixture)?;
        client.command(Command::Inspect)?;
        server.until(
            deadline,
            "X11 fullscreen property and geometry reply",
            |_| {
                client.poll()?;
                Ok(client.geometry.is_some())
            },
        )?;
        let expected = (server.area.size.w as u16, server.area.size.h as u16, true);
        if client.geometry != Some(expected) {
            return Err(format!(
                "X11 fullscreen mismatch: {:?}, expected {expected:?}",
                client.geometry
            )
            .into());
        }
        eprintln!(
            "satellite regression: committed fullscreen {:?}, X11 {:?}, imported elements={}",
            server.area, client.geometry, server.scene_size
        );
        Ok(())
    })();

    // Run teardown even when a regression condition fails. Killing our satellite also
    // releases an X11 thread blocked inside a reply; never touch any other display.
    let _ = client.command(Command::Stop);
    let shutdown = server.until(
        Instant::now() + Duration::from_secs(3),
        "X11 shutdown and Raven unmapping",
        |server| {
            client.poll()?;
            Ok(client.finished()
                && server.state.windows.is_empty()
                && server.state.space().elements().count() == 0)
        },
    );
    drop(clients);
    let waybar_shutdown = if fixture.pid().is_ok() {
        fixture.assert_stopped().and_then(|()| {
            server.until(
                Instant::now() + Duration::from_secs(3),
                "owned Waybar layer removed",
                |server| {
                    Ok(smithay::desktop::layer_map_for_output(
                        server.state.output.as_ref().ok_or("output disappeared")?,
                    )
                    .len()
                        == 0
                        && server.scene_size == 0)
                },
            )
        })
    } else {
        Ok(())
    };
    let joined = client.join();
    drop(server);
    let removed = sockets.assert_removed();
    let generated = fixture.cleanup();
    eprintln!(
        "satellite regression cleanup: unmap={shutdown:?}, client={joined:?}, sockets={removed:?}"
    );
    eprintln!(
        "Waybar regression cleanup: child/layer={waybar_shutdown:?}, generated files={generated:?}"
    );
    result?;
    waybar_shutdown?;
    generated?;
    shutdown?;
    joined?;
    removed?;
    Ok(())
}
