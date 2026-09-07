use super::{assertions::*, fixture::*, protocol::LayerClient};
use smithay::{
    desktop::layer_map_for_output,
    reexports::{
        wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1::{
            Anchor, KeyboardInteractivity,
        },
        wayland_server::Resource,
    },
};
use std::time::Duration;

#[test]
fn fuzzel_exclusive_focus_survives_windows_and_workspaces_then_restores_window() {
    let (mut f, shell) = fixture();
    let buffer = f.buffer();
    let (first, window) = map_window(&mut f, buffer);
    focus(&f, first.surface);
    // Put actual window pixels beneath the centered launcher, not just its tile.
    f.state
        .space_mut()
        .map_element(window.clone(), (350, 250), false);
    hit(&f, (360.0, 260.0), first.surface);
    f.state.pointer_location = (360.0, 260.0).into();

    let launcher = LayerClient::new(&mut f, shell);
    launcher.settings(&mut f, Anchor::empty(), 0, KeyboardInteractivity::Exclusive);
    let serial = launcher.configure(&mut f);
    mapped(&f, launcher, false);
    focus(&f, first.surface);
    launcher.ack(&mut f, serial);
    mapped(&f, launcher, false);
    launcher.attach(&mut f, buffer);
    mapped(&f, launcher, true);
    let geometry = layer_map_for_output(f.state.output.as_ref().unwrap())
        .layer_geometry(&layer(&f, launcher))
        .unwrap();
    assert_eq!(
        geometry,
        smithay::utils::Rectangle::new((350, 250).into(), (100, 100).into())
    );
    // Mapping may retile windows. Restore the overlap to test stacking explicitly.
    f.state
        .space_mut()
        .map_element(window.clone(), (350, 250), false);
    hit(&f, (360.0, 260.0), launcher.surface);
    assert_eq!(
        f.state
            .seat
            .get_pointer()
            .unwrap()
            .current_focus()
            .unwrap()
            .id()
            .protocol_id(),
        launcher.surface
    );
    focus(&f, launcher.surface);
    let callback = launcher.frame(&mut f);
    f.state.send_frames(Duration::from_millis(10));
    assert!(frame_done(&f.dispatch(), callback));

    let (second, _) = map_window(&mut f, buffer);
    focus(&f, launcher.surface);
    f.state.focus_window_on_motion((10.0, 10.0).into());
    focus(&f, launcher.surface);
    f.state.focus_window_at((10.0, 10.0).into());
    focus(&f, launcher.surface);
    f.state.switch_workspace(1);
    focus(&f, launcher.surface);
    hit(&f, (360.0, 260.0), launcher.surface);
    let callback = launcher.frame(&mut f);
    f.state.send_frames(Duration::from_millis(20));
    assert!(frame_done(&f.dispatch(), callback));
    f.state.switch_workspace(0);
    focus(&f, launcher.surface);

    // Restoring the saved window must not fall back to the topmost window.
    f.state.space_mut().raise_element(&window, false);
    f.wire.request(launcher.role, 7, &[]);
    f.wire.request(launcher.surface, 0, &[]);
    f.dispatch();
    f.state.refresh();
    assert!(f.state.layers.surfaces.is_empty());
    assert_eq!(
        layer_map_for_output(f.state.output.as_ref().unwrap()).len(),
        0
    );
    focus(&f, second.surface);
}
