use super::{assertions::*, fixture::*, protocol::LayerClient};
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1::{
    Anchor, KeyboardInteractivity,
};

#[test]
fn committed_keyboard_interactivity_distinguishes_hover_click_and_exclusive() {
    let (mut f, shell) = fixture();
    let buffer = f.buffer();
    let (top, _) = map_window(&mut f, buffer);
    let launcher = LayerClient::new(&mut f, shell);
    launcher.settings(&mut f, Anchor::empty(), 0, KeyboardInteractivity::None);
    let serial = launcher.configure(&mut f);
    launcher.ack(&mut f, serial);
    launcher.attach(&mut f, buffer);
    hit(&f, (360.0, 260.0), launcher.surface);
    f.state.focus_window_at((360.0, 260.0).into());
    focus(&f, top.surface);

    launcher.keyboard(&mut f, KeyboardInteractivity::OnDemand);
    launcher.commit(&mut f);
    f.state.focus_window_on_motion((360.0, 260.0).into());
    focus(&f, top.surface);
    f.state.focus_window_at((360.0, 260.0).into());
    focus(&f, launcher.surface);
    f.state.focus_window_at((10.0, 10.0).into());
    focus(&f, top.surface);

    launcher.keyboard(&mut f, KeyboardInteractivity::Exclusive);
    // Requests are double-buffered: processing the setter alone cannot steal focus.
    f.dispatch();
    f.state.refresh_layer_focus();
    focus(&f, top.surface);
    launcher.commit(&mut f);
    focus(&f, launcher.surface);
    launcher.keyboard(&mut f, KeyboardInteractivity::None);
    launcher.commit(&mut f);
    focus(&f, top.surface);
    f.state.focus_window_at((360.0, 260.0).into());
    focus(&f, top.surface);
}
