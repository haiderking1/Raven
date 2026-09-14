use super::fixture::{app, fixture};
use crate::{
    desktop::tests::wire::{string, word},
    state::State,
};
use smithay::{
    reexports::calloop::EventLoop, wayland::selection::data_device::set_data_device_selection,
};
use std::{
    io::Read,
    os::fd::{AsFd, FromRawFd, OwnedFd},
    sync::Arc,
    time::{Duration, Instant},
};
fn pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [0; 2];
    assert_eq!(
        unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) },
        0
    );
    unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) }
}
#[test]
fn png_clipboard_offer_survives_replacement_and_a_stalled_reader_does_not_block_dispatch() {
    let mut f = fixture();
    let mut event_loop = EventLoop::<State>::try_new().unwrap();
    f.state.install_screenshot(event_loop.handle());
    let registry = f.id();
    f.wire.request(1, 1, &[registry]);
    let globals = f.dispatch();
    let mut bind = |interface: &str, version: u32| {
        let global = globals
            .iter()
            .find(|e| {
                e.object == registry && e.opcode == 0 && {
                    let n = word(&e.args[4..]) as usize;
                    &e.args[8..8 + n - 1] == interface.as_bytes()
                }
            })
            .unwrap();
        let id = f.id();
        let mut args = word(&global.args).to_ne_bytes().to_vec();
        args.extend(string(interface));
        args.extend(version.to_ne_bytes());
        args.extend(id.to_ne_bytes());
        f.wire.bytes(registry, 0, &args, None);
        id
    };
    let seat = bind("wl_seat", 5);
    let manager = bind("wl_data_device_manager", 3);
    let device = f.id();
    f.wire.request(manager, 1, &[device, seat]);
    f.dispatch();
    app(&mut f, "editor");
    let mut rgba = Vec::new();
    let mut random = 1u32;
    for _ in 0..256 * 256 {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        rgba.extend([random as u8, (random >> 8) as u8, (random >> 16) as u8, 255]);
    }
    let png: Arc<[u8]> = Arc::from(super::super::native::encode(&rgba, 256, 256).unwrap());
    set_data_device_selection(
        &f.state.display_handle,
        &f.state.seat,
        vec!["image/png".into()],
        Some(png.clone()),
    );
    let events = f.dispatch();
    let offer = word(
        &events
            .iter()
            .find(|e| e.object == device && e.opcode == 5 && word(&e.args) != 0)
            .expect("clipboard selection offer")
            .args,
    );
    assert!(
        events
            .iter()
            .any(|e| e.object == offer && e.opcode == 0 && e.args == string("image/png"))
    );
    let (read, write) = pipe();
    f.wire
        .bytes(offer, 1, &string("image/png"), Some(write.as_fd()));
    drop(write);
    f.dispatch();
    event_loop
        .dispatch(Some(Duration::ZERO), &mut f.state)
        .unwrap();
    assert_eq!(
        f.state.screenshot.transfers, 1,
        "large image must be pending on the unread pipe"
    );
    let replacement = Arc::from(super::super::native::encode(&[0, 0, 0, 255], 1, 1).unwrap());
    set_data_device_selection(
        &f.state.display_handle,
        &f.state.seat,
        vec!["image/png".into()],
        Some(replacement),
    );
    f.dispatch();
    let mut read = std::fs::File::from(read);
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    let deadline = Instant::now() + Duration::from_secs(3);
    while bytes.len() < png.len() {
        assert!(Instant::now() < deadline, "clipboard transfer timed out");
        event_loop
            .dispatch(Some(Duration::ZERO), &mut f.state)
            .unwrap();
        loop {
            match read.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("{e}"),
            }
        }
    }
    event_loop
        .dispatch(Some(Duration::ZERO), &mut f.state)
        .unwrap();
    assert_eq!(bytes.as_slice(), png.as_ref());
    assert_eq!(f.state.screenshot.transfers, 0);
    let memory = smithay::reexports::rustix::fs::memfd_create(
        c"raven-screenshot-clipboard",
        smithay::reexports::rustix::fs::MemfdFlags::CLOEXEC,
    )
    .unwrap();
    let mut memory = std::fs::File::from(memory);
    f.wire
        .bytes(offer, 1, &string("image/png"), Some(memory.as_fd()));
    f.dispatch();
    let deadline = Instant::now() + Duration::from_secs(3);
    while f.state.screenshot.transfers != 0 {
        assert!(Instant::now() < deadline);
        f.state.refresh_screenshot();
        std::thread::yield_now();
    }
    std::io::Seek::rewind(&mut memory).unwrap();
    let mut file_bytes = Vec::new();
    memory.read_to_end(&mut file_bytes).unwrap();
    assert_eq!(file_bytes.as_slice(), png.as_ref());
    let (read, write) = pipe();
    drop(read);
    f.wire
        .bytes(offer, 1, &string("image/png"), Some(write.as_fd()));
    drop(write);
    f.dispatch();
    event_loop
        .dispatch(Some(Duration::ZERO), &mut f.state)
        .unwrap();
    assert_eq!(f.state.screenshot.transfers, 0);
}
