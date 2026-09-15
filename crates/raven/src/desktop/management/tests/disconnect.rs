use super::fixture::Harness;
use crate::{
    desktop::{
        management::minimize,
        tests::wire::{Wire, string, word},
    },
    state::ClientState,
};
use std::{os::unix::net::UnixStream, sync::Arc};
#[test]
fn losing_the_taskbar_connection_restores_windows_without_disconnecting_apps() {
    let mut h = Harness::new(3);
    let (_, window, original) = h.app("survivor");
    let (server, client) = UnixStream::pair().unwrap();
    h.f.state
        .display_handle
        .insert_client(server, Arc::new(ClientState::default()))
        .unwrap();
    let mut taskbar = Wire::new(client);
    taskbar.request(1, 1, &[2]);
    h.f.dispatch();
    let globals = taskbar.events();
    let global = globals
        .iter()
        .find(|e| {
            e.object == 2 && e.opcode == 0 && {
                let n = word(&e.args[4..]) as usize;
                &e.args[8..8 + n - 1] == b"zwlr_foreign_toplevel_manager_v1"
            }
        })
        .unwrap();
    let mut args = word(&global.args).to_ne_bytes().to_vec();
    args.extend(string("zwlr_foreign_toplevel_manager_v1"));
    args.extend(3u32.to_ne_bytes());
    args.extend(3u32.to_ne_bytes());
    taskbar.bytes(2, 0, &args, None);
    h.f.dispatch();
    let events = taskbar.events();
    let handle = word(
        &events
            .iter()
            .find(|e| e.object == 3 && e.opcode == 0)
            .unwrap()
            .args,
    );
    h.request(h.manager, 0);
    h.request(original, 7);
    assert!(h.f.state.foreign_toplevel.has_managers());
    taskbar.request(handle, 2, &[]);
    h.settle();
    assert!(minimize::hidden(&window));
    taskbar.socket.shutdown(std::net::Shutdown::Both).unwrap();
    h.settle();
    assert!(!minimize::hidden(&window));
    assert!(h.f.state.window_is_visible(&window));
    assert_eq!(h.f.state.windows.len(), 1);
    assert!(!h.f.state.foreign_toplevel.has_managers());
}
