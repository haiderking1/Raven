use smithay::reexports::calloop::LoopSignal;
use std::{
    io::Write,
    os::fd::{AsRawFd, OwnedFd},
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
};
pub(super) fn regular(fd: &OwnedFd) -> bool {
    let mut info = unsafe { std::mem::zeroed::<libc::stat>() };
    unsafe {
        libc::fstat(fd.as_raw_fd(), &mut info) == 0 && info.st_mode & libc::S_IFMT == libc::S_IFREG
    }
}
pub(in crate::desktop::screenshot) struct FileTransfer {
    pub bytes: usize,
    done: Receiver<()>,
}
impl FileTransfer {
    pub fn finished(&self) -> bool {
        !matches!(self.done.try_recv(), Err(mpsc::TryRecvError::Empty))
    }
}
pub(super) fn start(
    fd: OwnedFd,
    png: Arc<[u8]>,
    signal: LoopSignal,
) -> std::io::Result<FileTransfer> {
    let bytes = png.len();
    let (tx, done) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("raven-clipboard-file".into())
        .spawn(move || {
            let _ = std::fs::File::from(fd).write_all(&png);
            let _ = tx.send(());
            signal.wakeup();
        })?;
    Ok(FileTransfer { bytes, done })
}
