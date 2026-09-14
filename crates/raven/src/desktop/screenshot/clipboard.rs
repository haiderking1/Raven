mod files;
use crate::state::State;
pub(super) use files::FileTransfer;
use smithay::reexports::calloop::{Interest, Mode, PostAction, generic::Generic};
use std::{
    os::fd::{AsRawFd, OwnedFd},
    sync::Arc,
};
impl State {
    pub(crate) fn send_screenshot_clipboard(&mut self, fd: OwnedFd, png: Arc<[u8]>) {
        let Some(handle) = &self.screenshot.handle else {
            return;
        };
        if self.screenshot.transfers >= 16
            || self.screenshot.transfer_bytes.saturating_add(png.len()) > 512 * 1024 * 1024
        {
            return;
        }
        let bytes = png.len();
        if files::regular(&fd) {
            if let Ok(transfer) = files::start(fd, png, self.loop_signal.clone()) {
                self.screenshot.file_transfers.push(transfer);
                self.screenshot.transfers += 1;
                self.screenshot.transfer_bytes += bytes;
            }
            return;
        }
        let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            return;
        }
        let mut offset = 0;
        let result = handle.insert_source(
            Generic::new(fd, Interest::WRITE, Mode::Level),
            move |_, fd, state| {
                // Bound each dispatch; slow readers must never block input or frames.
                for _ in 0..4 {
                    let length = (png.len() - offset).min(65536);
                    if length == 0 {
                        state.screenshot.transfers -= 1;
                        state.screenshot.transfer_bytes -= bytes;
                        return Ok(PostAction::Remove);
                    }
                    let written = unsafe {
                        libc::write(fd.as_raw_fd(), png[offset..].as_ptr().cast(), length)
                    };
                    if written > 0 {
                        offset += written as usize;
                        continue;
                    }
                    let error = std::io::Error::last_os_error();
                    if written < 0 && error.kind() == std::io::ErrorKind::WouldBlock {
                        return Ok(PostAction::Continue);
                    }
                    if written < 0 && error.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    state.screenshot.transfers -= 1;
                    state.screenshot.transfer_bytes -= bytes;
                    return Ok(PostAction::Remove);
                }
                Ok(PostAction::Continue)
            },
        );
        if result.is_ok() {
            self.screenshot.transfers += 1;
            self.screenshot.transfer_bytes += bytes;
        }
    }
}

impl State {
    pub(super) fn refresh_clipboard_transfers(&mut self) {
        let mut count = 0;
        let mut bytes = 0;
        self.screenshot.file_transfers.retain(|transfer| {
            if transfer.finished() {
                count += 1;
                bytes += transfer.bytes;
                false
            } else {
                true
            }
        });
        self.screenshot.transfers -= count;
        self.screenshot.transfer_bytes -= bytes;
    }
}
